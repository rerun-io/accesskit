// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![allow(non_upper_case_globals)]

use accesskit_consumer::{
    Node, TextPosition as Position, TextRange as Range, Tree, TreeState, WeakTextRange as WeakRange,
};
use parking_lot::RwLock;
use std::sync::{Arc, Weak};
use windows::{
    core::*,
    Win32::{Foundation::*, System::Com::*, UI::Accessibility::*},
};

use crate::{node::PlatformNode, util::*};

fn upgrade_range<'a>(weak: &WeakRange, tree_state: &'a TreeState) -> Result<Range<'a>> {
    if let Some(range) = weak.upgrade(tree_state) {
        Ok(range)
    } else {
        Err(element_not_available())
    }
}

fn upgrade_range_node<'a>(weak: &WeakRange, tree_state: &'a TreeState) -> Result<Node<'a>> {
    if let Some(node) = weak.upgrade_node(tree_state) {
        Ok(node)
    } else {
        Err(element_not_available())
    }
}

fn weak_comparable_position_from_endpoint(
    range: &WeakRange,
    endpoint: TextPatternRangeEndpoint,
) -> Result<&(Vec<usize>, usize)> {
    match endpoint {
        TextPatternRangeEndpoint_Start => Ok(range.start_comparable()),
        TextPatternRangeEndpoint_End => Ok(range.end_comparable()),
        _ => Err(invalid_arg()),
    }
}

fn position_from_endpoint<'a>(
    range: &Range<'a>,
    endpoint: TextPatternRangeEndpoint,
) -> Result<Position<'a>> {
    match endpoint {
        TextPatternRangeEndpoint_Start => Ok(range.start()),
        TextPatternRangeEndpoint_End => Ok(range.end()),
        _ => Err(invalid_arg()),
    }
}

fn set_endpoint_position<'a>(
    range: &mut Range<'a>,
    endpoint: TextPatternRangeEndpoint,
    pos: Position<'a>,
) -> Result<()> {
    match endpoint {
        TextPatternRangeEndpoint_Start => {
            range.set_start(pos);
        }
        TextPatternRangeEndpoint_End => {
            range.set_end(pos);
        }
        _ => {
            return Err(invalid_arg());
        }
    }
    Ok(())
}

fn back_to_unit_start(start: Position, unit: TextUnit) -> Result<Position> {
    match unit {
        TextUnit_Character => {
            // If we get here, this position is at the start of a non-degenerate
            // range, so it's always at the start of a character.
            debug_assert!(!start.is_document_end());
            Ok(start)
        }
        TextUnit_Format => {
            if start.is_format_start() {
                Ok(start)
            } else {
                Ok(start.backward_to_format_start())
            }
        }
        TextUnit_Word => {
            if start.is_word_start() {
                Ok(start)
            } else {
                Ok(start.backward_to_word_start())
            }
        }
        TextUnit_Line => {
            if start.is_line_start() {
                Ok(start)
            } else {
                Ok(start.backward_to_line_start())
            }
        }
        TextUnit_Paragraph => {
            if start.is_paragraph_start() {
                Ok(start)
            } else {
                Ok(start.backward_to_paragraph_start())
            }
        }
        TextUnit_Page => {
            if start.is_page_start() {
                Ok(start)
            } else {
                Ok(start.backward_to_page_start())
            }
        }
        TextUnit_Document => {
            if start.is_document_start() {
                Ok(start)
            } else {
                Ok(start.document_start())
            }
        }
        _ => Err(invalid_arg()),
    }
}

fn move_forward_to_start(pos: Position, unit: TextUnit) -> Result<Position> {
    match unit {
        TextUnit_Character => Ok(pos.forward_to_character_start()),
        TextUnit_Format => Ok(pos.forward_to_format_start()),
        TextUnit_Word => Ok(pos.forward_to_word_start()),
        TextUnit_Line => Ok(pos.forward_to_line_start()),
        TextUnit_Paragraph => Ok(pos.forward_to_paragraph_start()),
        TextUnit_Page => Ok(pos.forward_to_page_start()),
        TextUnit_Document => Ok(pos.document_end()),
        _ => Err(invalid_arg()),
    }
}

fn move_forward_to_end(pos: Position, unit: TextUnit) -> Result<Position> {
    match unit {
        TextUnit_Character => Ok(pos.forward_to_character_end()),
        TextUnit_Format => Ok(pos.forward_to_format_end()),
        TextUnit_Word => Ok(pos.forward_to_word_end()),
        TextUnit_Line => Ok(pos.forward_to_line_end()),
        TextUnit_Paragraph => Ok(pos.forward_to_paragraph_end()),
        TextUnit_Page => Ok(pos.forward_to_page_end()),
        TextUnit_Document => Ok(pos.document_end()),
        _ => Err(invalid_arg()),
    }
}

fn move_backward(pos: Position, unit: TextUnit) -> Result<Position> {
    match unit {
        TextUnit_Character => Ok(pos.backward_to_character_start()),
        TextUnit_Format => Ok(pos.backward_to_format_start()),
        TextUnit_Word => Ok(pos.backward_to_word_start()),
        TextUnit_Line => Ok(pos.backward_to_line_start()),
        TextUnit_Paragraph => Ok(pos.backward_to_paragraph_start()),
        TextUnit_Page => Ok(pos.backward_to_page_start()),
        TextUnit_Document => Ok(pos.document_start()),
        _ => Err(invalid_arg()),
    }
}

fn move_position(
    mut pos: Position,
    unit: TextUnit,
    to_end: bool,
    count: i32,
) -> Result<(Position, i32)> {
    let forward = count > 0;
    let count = count.abs();
    let mut moved = 0i32;
    for _ in 0..count {
        let at_end = if forward {
            pos.is_document_end()
        } else {
            pos.is_document_start()
        };
        if at_end {
            break;
        }
        pos = if forward {
            if to_end {
                move_forward_to_end(pos, unit)
            } else {
                move_forward_to_start(pos, unit)
            }
        } else {
            move_backward(pos, unit)
        }?;
        moved += 1;
    }
    if !forward {
        moved = -moved;
    }
    Ok((pos, moved))
}

#[implement(ITextRangeProvider)]
pub(crate) struct PlatformRange {
    tree: Weak<Tree>,
    state: RwLock<WeakRange>,
    hwnd: HWND,
}

impl PlatformRange {
    pub(crate) fn new(tree: &Weak<Tree>, range: Range, hwnd: HWND) -> Self {
        Self {
            tree: tree.clone(),
            state: RwLock::new(range.downgrade()),
            hwnd,
        }
    }

    fn upgrade_tree(&self) -> Result<Arc<Tree>> {
        if let Some(tree) = self.tree.upgrade() {
            Ok(tree)
        } else {
            Err(element_not_available())
        }
    }

    fn with_tree_state<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&TreeState) -> Result<T>,
    {
        let tree = self.upgrade_tree()?;
        let state = tree.read();
        f(&state)
    }

    fn upgrade_node<'a>(&self, tree_state: &'a TreeState) -> Result<Node<'a>> {
        let state = self.state.read();
        upgrade_range_node(&state, tree_state)
    }

    fn with_node<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(Node) -> Result<T>,
    {
        self.with_tree_state(|tree_state| {
            let node = self.upgrade_node(tree_state)?;
            f(node)
        })
    }

    fn upgrade_for_read<'a>(&self, tree_state: &'a TreeState) -> Result<Range<'a>> {
        let state = self.state.read();
        upgrade_range(&state, tree_state)
    }

    fn read<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(Range) -> Result<T>,
    {
        self.with_tree_state(|tree_state| {
            let range = self.upgrade_for_read(tree_state)?;
            f(range)
        })
    }

    fn write<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Range) -> Result<T>,
    {
        self.with_tree_state(|tree_state| {
            let mut state = self.state.write();
            let mut range = upgrade_range(&state, tree_state)?;
            let result = f(&mut range);
            *state = range.downgrade();
            result
        })
    }

    fn action<F>(&self, f: F) -> Result<()>
    where
        for<'a> F: FnOnce(&'a Tree, Range<'a>) -> Result<()>,
    {
        let tree = self.upgrade_tree()?;
        let tree_state = tree.read();
        let range = self.upgrade_for_read(&tree_state)?;
        f(&tree, range)
    }

    fn require_same_tree(&self, other: &PlatformRange) -> Result<()> {
        if self.tree.ptr_eq(&other.tree) {
            Ok(())
        } else {
            Err(invalid_arg())
        }
    }
}

impl Clone for PlatformRange {
    fn clone(&self) -> Self {
        PlatformRange {
            tree: self.tree.clone(),
            state: RwLock::new(self.state.read().clone()),
            hwnd: self.hwnd,
        }
    }
}

// Some text range methods take another text range interface pointer as a
// parameter. We need to cast these interface pointers to their underlying
// implementations. We assume that AccessKit is the only UIA provider
// within this process. This seems a safe assumption for most AccessKit users.

impl ITextRangeProvider_Impl for PlatformRange {
    fn Clone(&self) -> Result<ITextRangeProvider> {
        Ok(self.clone().into())
    }

    fn Compare(&self, other: &Option<ITextRangeProvider>) -> Result<BOOL> {
        let other = required_param(other)?.as_impl();
        Ok((self.tree.ptr_eq(&other.tree) && *self.state.read() == *other.state.read()).into())
    }

    fn CompareEndpoints(
        &self,
        endpoint: TextPatternRangeEndpoint,
        other: &Option<ITextRangeProvider>,
        other_endpoint: TextPatternRangeEndpoint,
    ) -> Result<i32> {
        let other = required_param(other)?.as_impl();
        if std::ptr::eq(other as *const _, self as *const _) {
            // Comparing endpoints within the same range can be done
            // safely without upgrading the range. This allows ATs
            // to determine whether an old range is degenerate even if
            // that range is no longer valid.
            let state = self.state.read();
            let other_state = other.state.read();
            let pos = weak_comparable_position_from_endpoint(&state, endpoint)?;
            let other_pos = weak_comparable_position_from_endpoint(&other_state, other_endpoint)?;
            let result = pos.cmp(other_pos);
            return Ok(result as i32);
        }
        self.require_same_tree(other)?;
        self.with_tree_state(|tree_state| {
            let range = self.upgrade_for_read(tree_state)?;
            let other_range = other.upgrade_for_read(tree_state)?;
            if range.node().id() != other_range.node().id() {
                return Err(invalid_arg());
            }
            let pos = position_from_endpoint(&range, endpoint)?;
            let other_pos = position_from_endpoint(&other_range, other_endpoint)?;
            let result = pos.partial_cmp(&other_pos).unwrap();
            Ok(result as i32)
        })
    }

    fn ExpandToEnclosingUnit(&self, unit: TextUnit) -> Result<()> {
        if unit == TextUnit_Document {
            // Handle document as a special case so we can get to a document
            // range even if the current endpoints are now invalid.
            // Based on observed behavior, Narrator needs this ability.
            return self.with_tree_state(|tree_state| {
                let mut state = self.state.write();
                let node = upgrade_range_node(&state, tree_state)?;
                *state = node.document_range().downgrade();
                Ok(())
            });
        }
        self.write(|range| {
            let start = range.start();
            if unit == TextUnit_Character && start.is_document_end() {
                // We know from experimentation that some Windows ATs
                // expect ExpandToEnclosingUnit(TextUnit_Character)
                // to do nothing if the range is degenerate at the end
                // of the document.
                return Ok(());
            }
            let start = back_to_unit_start(start, unit)?;
            range.set_start(start);
            if !start.is_document_end() {
                let end = move_forward_to_end(start, unit)?;
                range.set_end(end);
            }
            Ok(())
        })
    }

    fn FindAttribute(
        &self,
        _id: i32,
        _value: &VARIANT,
        _backward: BOOL,
    ) -> Result<ITextRangeProvider> {
        // TODO: implement when we support variable formatting (part of rich text)
        // Justification: JUCE doesn't implement this.
        Err(Error::OK)
    }

    fn FindText(
        &self,
        _text: &BSTR,
        _backward: BOOL,
        _ignore_case: BOOL,
    ) -> Result<ITextRangeProvider> {
        // TODO: implement when there's a real-world use case that requires it
        // Justification: Quorum doesn't implement this and is being used
        // by blind students.
        Err(Error::OK)
    }

    fn GetAttributeValue(&self, id: i32) -> Result<VARIANT> {
        let id = UIA_TEXTATTRIBUTE_ID(id as _);
        match id {
            UIA_IsReadOnlyAttributeId => {
                // TBD: do we ever want to support mixed read-only/editable text?
                self.with_node(|node| {
                    let value = node.is_read_only();
                    Ok(VariantFactory::from(value).into())
                })
            }
            UIA_CaretPositionAttributeId => self.read(|range| {
                let mut value = CaretPosition_Unknown;
                if range.is_degenerate() {
                    let pos = range.start();
                    if pos.is_line_start() {
                        value = CaretPosition_BeginningOfLine;
                    } else if pos.is_line_end() {
                        value = CaretPosition_EndOfLine;
                    }
                }
                Ok(VariantFactory::from(value).into())
            }),
            // TODO: implement more attributes
            _ => {
                let value = unsafe { UiaGetReservedNotSupportedValue() }.unwrap();
                Ok(VariantFactory::from(value).into())
            }
        }
    }

    fn GetBoundingRectangles(&self) -> Result<*mut SAFEARRAY> {
        self.read(|range| {
            let rects = range.bounding_boxes();
            if rects.is_empty() {
                return Ok(std::ptr::null_mut());
            }
            let client_top_left = client_top_left(self.hwnd);
            let mut result = Vec::<f64>::new();
            result.reserve(rects.len() * 4);
            for rect in rects {
                result.push(rect.x0 + client_top_left.x);
                result.push(rect.y0 + client_top_left.y);
                result.push(rect.width());
                result.push(rect.height());
            }
            Ok(safe_array_from_f64_slice(&result))
        })
    }

    fn GetEnclosingElement(&self) -> Result<IRawElementProviderSimple> {
        self.with_node(|node| {
            // Revisit this if we eventually support embedded objects.
            Ok(PlatformNode {
                tree: self.tree.clone(),
                node_id: node.id(),
                hwnd: self.hwnd,
            }
            .into())
        })
    }

    fn GetText(&self, _max_length: i32) -> Result<BSTR> {
        // The Microsoft docs imply that the provider isn't _required_
        // to truncate text at the max length, so we just ignore it.
        self.read(|range| Ok(range.text().into()))
    }

    fn Move(&self, unit: TextUnit, count: i32) -> Result<i32> {
        self.write(|range| {
            let degenerate = range.is_degenerate();
            let start = range.start();
            let start = if degenerate {
                start
            } else {
                back_to_unit_start(start, unit)?
            };
            let (start, moved) = move_position(start, unit, false, count)?;
            if moved != 0 {
                range.set_start(start);
                let end = if degenerate || start.is_document_end() {
                    start
                } else {
                    move_forward_to_end(start, unit)?
                };
                range.set_end(end);
            }
            Ok(moved)
        })
    }

    fn MoveEndpointByUnit(
        &self,
        endpoint: TextPatternRangeEndpoint,
        unit: TextUnit,
        count: i32,
    ) -> Result<i32> {
        self.write(|range| {
            let pos = position_from_endpoint(range, endpoint)?;
            let (pos, moved) =
                move_position(pos, unit, endpoint == TextPatternRangeEndpoint_End, count)?;
            set_endpoint_position(range, endpoint, pos)?;
            Ok(moved)
        })
    }

    fn MoveEndpointByRange(
        &self,
        endpoint: TextPatternRangeEndpoint,
        other: &Option<ITextRangeProvider>,
        other_endpoint: TextPatternRangeEndpoint,
    ) -> Result<()> {
        let other = required_param(other)?.as_impl();
        self.require_same_tree(other)?;
        // We have to obtain the tree state and ranges manually to avoid
        // lifetime issues, and work with the two locks in a specific order
        // to avoid deadlock.
        self.with_tree_state(|tree_state| {
            let other_range = other.upgrade_for_read(tree_state)?;
            let mut state = self.state.write();
            let mut range = upgrade_range(&state, tree_state)?;
            if range.node().id() != other_range.node().id() {
                return Err(invalid_arg());
            }
            let pos = position_from_endpoint(&other_range, other_endpoint)?;
            set_endpoint_position(&mut range, endpoint, pos)?;
            *state = range.downgrade();
            Ok(())
        })
    }

    fn Select(&self) -> Result<()> {
        self.action(|tree, range| {
            tree.select_text_range(&range);
            Ok(())
        })
    }

    fn AddToSelection(&self) -> Result<()> {
        // AccessKit doesn't support multiple text selections.
        Err(invalid_operation())
    }

    fn RemoveFromSelection(&self) -> Result<()> {
        // AccessKit doesn't support multiple text selections.
        Err(invalid_operation())
    }

    fn ScrollIntoView(&self, align_to_top: BOOL) -> Result<()> {
        self.action(|tree, range| {
            let position = if align_to_top.into() {
                range.start()
            } else {
                range.end()
            };
            tree.scroll_text_position_into_view(&position);
            Ok(())
        })
    }

    fn GetChildren(&self) -> Result<*mut SAFEARRAY> {
        // We don't support embedded objects in text.
        Ok(safe_array_from_com_slice(&[]))
    }
}

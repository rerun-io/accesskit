// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::OwnedObjectAddress;
use accesskit::kurbo::{Point, Rect};
use atspi::CoordType;

pub(crate) struct AppContext {
    pub(crate) name: String,
    pub(crate) toolkit_name: String,
    pub(crate) toolkit_version: String,
    pub(crate) id: Option<i32>,
    pub(crate) desktop_address: Option<OwnedObjectAddress>,
}

impl AppContext {
    pub(crate) fn new(name: String, toolkit_name: String, toolkit_version: String) -> Self {
        Self {
            name,
            toolkit_name,
            toolkit_version,
            id: None,
            desktop_address: None,
        }
    }
}

#[derive(Default)]
pub(crate) struct WindowBounds {
    pub(crate) outer: Rect,
    pub(crate) inner: Rect,
}

impl WindowBounds {
    pub(crate) fn top_left(&self, coord_type: CoordType, is_root: bool) -> Point {
        match coord_type {
            CoordType::Screen if is_root => self.outer.origin(),
            CoordType::Screen => self.inner.origin(),
            CoordType::Window if is_root => Point::ZERO,
            CoordType::Window => {
                let outer_position = self.outer.origin();
                let inner_position = self.inner.origin();
                Point::new(
                    inner_position.x - outer_position.x,
                    inner_position.y - outer_position.y,
                )
            }
            _ => unimplemented!(),
        }
    }
}

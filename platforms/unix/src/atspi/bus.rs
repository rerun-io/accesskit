// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    atspi::{interfaces::*, object_address::*, ObjectId},
    PlatformRootNode,
};
use atspi::{bus::BusProxyBlocking, socket::SocketProxyBlocking, EventBody};
use serde::Serialize;
use std::{collections::HashMap, env::var};
use zbus::{
    blocking::{Connection, ConnectionBuilder},
    names::{BusName, InterfaceName, MemberName, OwnedUniqueName},
    zvariant::{ObjectPath, Str, Value},
    Address, Result,
};

#[derive(Clone)]
pub(crate) struct Bus {
    conn: Connection,
    socket_proxy: SocketProxyBlocking<'static>,
}

impl Bus {
    pub fn a11y_bus() -> Option<Self> {
        let conn = a11y_bus()?;
        let socket_proxy = SocketProxyBlocking::new(&conn).ok()?;
        Some(Bus { conn, socket_proxy })
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn unique_name(&self) -> &OwnedUniqueName {
        self.conn.unique_name().unwrap()
    }

    pub fn register_interface<T>(&self, path: &str, interface: T) -> Result<bool>
    where
        T: zbus::Interface,
    {
        self.conn.object_server().at(path, interface)
    }

    pub fn unregister_interface<T>(&self, path: &str) -> Result<bool>
    where
        T: zbus::Interface,
    {
        self.conn.object_server().remove::<T, _>(path)
    }

    pub fn register_root_node(&mut self, node: PlatformRootNode) -> Result<bool> {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, ObjectId::root().as_str());
        let registered = self
            .conn
            .object_server()
            .at(path.clone(), ApplicationInterface(node.clone()))?
            && self.conn.object_server().at(
                path,
                AccessibleInterface::new(self.unique_name().to_owned(), node.clone()),
            )?;
        if registered {
            let desktop = self.socket_proxy.embed(&(
                self.unique_name().as_str(),
                ObjectPath::from_str_unchecked(ROOT_PATH),
            ))?;
            if let Some(context) = node.context.upgrade() {
                context.write().desktop_address = Some(desktop.into());
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(crate) async fn emit_object_event(
        &self,
        target: ObjectId<'_>,
        event: ObjectEvent,
    ) -> Result<()> {
        let interface = "org.a11y.atspi.Event.Object";
        let signal = match event {
            ObjectEvent::BoundsChanged(_) => "BoundsChanged",
            ObjectEvent::ChildAdded(_, _) | ObjectEvent::ChildRemoved(_) => "ChildrenChanged",
            ObjectEvent::PropertyChanged(_) => "PropertyChange",
            ObjectEvent::StateChanged(_, _) => "StateChanged",
        };
        let properties = HashMap::new();
        match event {
            ObjectEvent::BoundsChanged(bounds) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: "",
                        detail1: 0,
                        detail2: 0,
                        any_data: Value::from(bounds),
                        properties,
                    },
                )
                .await
            }
            ObjectEvent::ChildAdded(index, child) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: "add",
                        detail1: index as i32,
                        detail2: 0,
                        any_data: child.into_value(self.unique_name().clone()),
                        properties,
                    },
                )
                .await
            }
            ObjectEvent::ChildRemoved(child) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: "remove",
                        detail1: -1,
                        detail2: 0,
                        any_data: child.into_value(self.unique_name().clone()),
                        properties,
                    },
                )
                .await
            }
            ObjectEvent::PropertyChanged(property) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: match property {
                            Property::Name(_) => "accessible-name",
                            Property::Description(_) => "accessible-description",
                            Property::Parent(_) => "accessible-parent",
                            Property::Role(_) => "accessible-role",
                            Property::Value(_) => "accessible-value",
                        },
                        detail1: 0,
                        detail2: 0,
                        any_data: match property {
                            Property::Name(value) => Str::from(value).into(),
                            Property::Description(value) => Str::from(value).into(),
                            Property::Parent(Some(parent)) => {
                                parent.into_value(self.unique_name().clone())
                            }
                            Property::Parent(None) => {
                                OwnedObjectAddress::root(self.unique_name().clone()).into()
                            }
                            Property::Role(value) => Value::U32(value as u32),
                            Property::Value(value) => Value::F64(value),
                        },
                        properties,
                    },
                )
                .await
            }
            ObjectEvent::StateChanged(state, value) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: state,
                        detail1: value as i32,
                        detail2: 0,
                        any_data: 0i32.into(),
                        properties,
                    },
                )
                .await
            }
        }
    }

    pub(crate) async fn emit_window_event(
        &self,
        target: ObjectId<'_>,
        window_name: String,
        event: WindowEvent,
    ) -> Result<()> {
        let signal = match event {
            WindowEvent::Activated => "Activate",
            WindowEvent::Deactivated => "Deactivate",
        };
        self.emit_event(
            target,
            "org.a11y.atspi.Event.Window",
            signal,
            EventBody {
                kind: "",
                detail1: 0,
                detail2: 0,
                any_data: window_name.into(),
                properties: HashMap::new(),
            },
        )
        .await
    }

    async fn emit_event<T: Serialize>(
        &self,
        id: ObjectId<'_>,
        interface: &str,
        signal_name: &str,
        body: EventBody<'_, T>,
    ) -> Result<()> {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, id.as_str());
        self.conn
            .inner()
            .emit_signal(
                Option::<BusName>::None,
                path,
                InterfaceName::from_str_unchecked(interface),
                MemberName::from_str_unchecked(signal_name),
                &body,
            )
            .await
    }
}

fn a11y_bus() -> Option<Connection> {
    let address = match var("AT_SPI_BUS_ADDRESS") {
        Ok(address) if !address.is_empty() => address,
        _ => {
            let session_bus = Connection::session().ok()?;
            BusProxyBlocking::new(&session_bus)
                .ok()?
                .get_address()
                .ok()?
        }
    };
    let address: Address = address.as_str().try_into().ok()?;
    ConnectionBuilder::address(address).ok()?.build().ok()
}

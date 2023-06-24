use serde::{Deserialize, Serialize};

pub const AF_INET: u8 = 2;

#[repr(u16)]
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum RouteAttrType {
    IfaUnspec = 0,
    IfaAddress = 1,
    IfaLocal = 2,
    IfaLabel = 3,
    IfaBroadcast = 4,
    IfaAnycast = 5,
    IfaCacheinfo = 6,
    IfaMulticast = 7,
}

impl From<RouteAttrType> for u16 {
    fn from(value: RouteAttrType) -> Self {
        value as u16
    }
}

#[repr(u16)]
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum RouteMessageType {
    // Link
    NewLink = 16,
    DelLink = 17,
    GetLink = 18,
    SetLink = 19,
    // Addr
    NewAddr = 20,
    DelAddr = 21,
    GetAddr = 22,
    // Route
    NewRoute = 24,
    DelRoute = 25,
    GetRoute = 26,
}

impl From<RouteMessageType> for u16 {
    fn from(value: RouteMessageType) -> Self {
        value as u16
    }
}

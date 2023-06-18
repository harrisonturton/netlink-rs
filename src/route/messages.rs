use serde::{Serialize, Deserialize};

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Header {
    pub family: u8,
    pub dst_len: u8,
    pub src_len: u8,
    pub tos: u8,
    pub table: u8,
    pub protocol: u8,
    pub scope: u8,
    pub typ: u8,
    pub flags: u8,
}

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Attr {
    pub len: u16,
    pub typ: u16,
}

pub mod flags {
    pub const GET_ROUTE: u32 = 0x1A;
}
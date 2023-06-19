use serde::{Deserialize, Serialize};
use crate::{Error, Result};


#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct RouteMessage {
    pub header: RouteHeader,
    pub attrs: Vec<Attr>,
}

impl RouteMessage {
    pub fn builder() -> RouteMessageBuilder {
        RouteMessageBuilder::new()
    }
}

pub struct RouteMessageBuilder {
    header: Option<RouteHeader>,
    attrs: Vec<Attr>,
}

impl RouteMessageBuilder {
    pub fn new() -> Self {
        Self {
            header: None,
            attrs: Vec::new(),
        }
    }

    pub fn header(mut self, header: RouteHeader) -> Self {
        self.header = Some(header);
        self
    }

    pub fn attrs(mut self, attrs: Vec<Attr>) -> Self {
        self.attrs = attrs;
        self
    }

    pub fn build(self) -> Result<RouteMessage> {
        let header = self
            .header
            .ok_or_else(|| Error::ErrMissingField("header".to_string()))?;
        Ok(RouteMessage {
            header,
            attrs: self.attrs,
        })
    }
}

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct RouteHeader {
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

impl RouteHeader {
    pub fn builder() -> RouteHeaderBuilder {
        RouteHeaderBuilder::new()
    }
}

pub struct RouteHeaderBuilder {
    header: RouteHeader,
}

impl RouteHeaderBuilder {
    pub fn new() -> Self {
        Self {
            header: RouteHeader::default(),
        }
    }

    pub fn family(mut self, family: u8) -> Self {
        self.header.family = family;
        self
    }

    pub fn dst_len(mut self, dst_len: u8) -> Self {
        self.header.dst_len = dst_len;
        self
    }

    pub fn tos(mut self, tos: u8) -> Self {
        self.header.tos = tos;
        self
    }

    pub fn table(mut self, table: u8) -> Self {
        self.header.table = table;
        self
    }

    pub fn scope(mut self, scope: u8) -> Self {
        self.header.scope = scope;
        self
    }

    pub fn typ(mut self, typ: u8) -> Self {
        self.header.typ = typ;
        self
    }

    pub fn flags(mut self, flags: u8) -> Self {
        self.header.flags = flags;
        self
    }

    pub fn build(self) -> RouteHeader {
        self.header
    }
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

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Addr {
    pub family: u8,
    pub prefixlen: u8,
    pub flags: u8,
    pub scope: u8,
    pub index: u32,
}

#[derive(PartialEq, Clone, Debug)]
pub enum MessageType {
    NEW_LINK = 16,
    NEW_ADDR = 20,
    DEL_ADDR = 21,
    GET_ADDR = 22,
    NEW_ROUTE = 24,
    DEL_ROUTE = 25,
    GET_ROUTE = 26,
    NEW_NEIGH = 28,
    DEL_NEIGH = 29,
    GET_NEIGH = 30,
    NEW_RULE = 32,
    DEL_RULE = 33,
    GET_RULE = 34,
    NEW_QDISC = 36,
    DEL_QDISC = 37,
    GET_QDISC = 38,
    NEW_TCLASS = 40,
    DEL_TCLASS = 41,
    GET_TCLASS = 42,
    NEW_TFILTER = 44,
    DEL_TFILTER = 45,
    GET_TFILTER = 46,
    NEW_ACTION = 48,
    DEL_ACTION = 49,
    GET_ACTION = 50,
    NEW_PREFIX = 52,
    GET_MULTICAST = 58,
    GET_ANYCAST = 62,
    NEW_NEIGH_TBL = 64,
    GET_NEIGH_TBL = 66,
    SET_NEIGH_TBL = 67,
    NEW_ND_USER_OPT = 68,
    NEW_ADDR_LABEL = 72,
    DEL_ADDR_LABEL = 73,
    GET_ADDR_LABEL = 74,
    GET_DCB = 78,
    SET_DCB = 79,
    NEW_NET_CONF = 80,
    GET_NET_CONF = 82,
    NEW_MDB = 84,
    DEL_MDB = 85,
    GET_MDB = 86,
    NEW_NSID = 88,
    DEL_NSID = 89,
    GET_NSID = 90,
}

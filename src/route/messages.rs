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

pub enum MessageType {
    NEWLINK = 16,
    DELLINK = 17,
    GETLINK = 18,
    SETLINK = 19,
    NEWADDR = 20,
    DELADDR = 21,
    GETADDR = 22,
    NEWROUTE = 24,
    DELROUTE = 25,
    GETROUTE = 26,
    NEWNEIGH = 28,
    DELNEIGH = 29,
    GETNEIGH = 30,
    NEWRULE = 32,
    DELRULE = 33,
    GETRULE = 34,
    NEWQDISC = 36,
    DELQDISC = 37,
    GETQDISC = 38,
    NEWTCLASS = 40,
    DELTCLASS = 41,
    GETTCLASS = 42,
    NEWTFILTER = 44,
    DELTFILTER = 45,
    GETTFILTER = 46,
    NEWACTION = 48,
    DELACTION = 49,
    GETACTION = 50,
    NEWPREFIX = 52,
    GETMULTICAST = 58,
    GETANYCAST = 62,
    NEWNEIGHTBL = 64,
    GETNEIGHTBL = 66,
    SETNEIGHTBL = 67,
    NEWNDUSEROPT = 68,
    NEWADDRLABEL = 72,
    DELADDRLABEL = 73,
    GETADDRLABEL = 74,
    GETDCB = 78,
    SETDCB = 79,
    NEWNETCONF = 80,
    GETNETCONF = 82,
    NEWMDB = 84,
    DELMDB = 85,
    GETMDB = 86,
    NEWNSID = 88,
    DELNSID = 89,
    GETNSID = 90,
}

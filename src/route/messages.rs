use super::RouteAttrType;
use crate::{aligned_size_of, Error, Result};
use bincode::serialize;
use serde::{Deserialize, Serialize};

// See https://man7.org/linux/man-pages/man7/rtnetlink.7.html
pub const IFINFO_CHANGE_PLACEHOLDER_MAGIC: u32 = 0xFFFFFFFF;

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct InterfaceInfoHeader {
    pub family: u8,
    pub typ: u16,
    pub index: i32,
    pub flags: u32,
    // Should always equal [IFINFO_CHANGE_PLACEHOLDER_MAGIC]
    #[allow(dead_code)]
    pub change: u32,
}

impl InterfaceInfoHeader {
    pub fn builder() -> InterfaceInfoHeaderBuilder {
        InterfaceInfoHeaderBuilder::new()
    }
}

#[derive(Default)]
pub struct InterfaceInfoHeaderBuilder {
    family: u8,
    typ: u16,
    index: i32,
    flags: u32,
    #[allow(dead_code)]
    change: u32,
}

impl InterfaceInfoHeaderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn family(mut self, family: u8) -> Self {
        self.family = family;
        self
    }

    pub fn typ(mut self, typ: u16) -> Self {
        self.typ = typ;
        self
    }

    pub fn index(mut self, index: i32) -> Self {
        self.index = index;
        self
    }

    pub fn flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }

    pub fn build(self) -> InterfaceInfoHeader {
        InterfaceInfoHeader {
            family: self.family,
            typ: self.typ,
            index: self.index,
            flags: self.flags,
            change: IFINFO_CHANGE_PLACEHOLDER_MAGIC,
        }
    }
}

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct InterfaceAddrHeader {
    pub family: u8,
    pub prefixlen: u8,
    pub flags: u8,
    pub scope: u8,
    pub index: u32,
}

impl InterfaceAddrHeader {
    pub fn builder() -> IfAddrMessageBuilder {
        IfAddrMessageBuilder::new()
    }
}

#[derive(Default)]
pub struct IfAddrMessageBuilder {
    family: Option<u8>,
    prefixlen: Option<u8>,
    flags: Option<u8>,
    scope: Option<u8>,
    index: Option<u32>,
}

impl IfAddrMessageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn family(mut self, family: u8) -> Self {
        self.family = Some(family);
        self
    }

    pub fn prefixlen(mut self, prefixlen: u8) -> Self {
        self.prefixlen = Some(prefixlen);
        self
    }

    pub fn flags(mut self, flags: u8) -> Self {
        self.flags = Some(flags);
        self
    }

    pub fn scope(mut self, scope: u8) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn index(mut self, index: u32) -> Self {
        self.index = Some(index);
        self
    }

    pub fn build(self) -> InterfaceAddrHeader {
        InterfaceAddrHeader {
            family: self.family.unwrap_or_default(),
            prefixlen: self.prefixlen.unwrap_or_default(),
            flags: self.flags.unwrap_or_default(),
            scope: self.scope.unwrap_or_default(),
            index: self.index.unwrap_or_default(),
        }
    }
}

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RouteMessage<H: Serialize> {
    pub header: H,
    pub attrs: Vec<RouteAttr>,
}

impl<H: Serialize> RouteMessage<H> {
    pub fn builder() -> RouteMessageBuilder<H> {
        RouteMessageBuilder::new()
    }
}

pub struct RouteMessageBuilder<H> {
    header: Option<H>,
    attrs: Vec<RouteAttr>,
}

impl<H: Serialize> RouteMessageBuilder<H> {
    pub fn new() -> Self {
        Self {
            header: None,
            attrs: Vec::new(),
        }
    }

    pub fn header(mut self, header: H) -> Self {
        self.header = Some(header);
        self
    }

    pub fn attrs(mut self, attrs: Vec<RouteAttr>) -> Self {
        self.attrs = attrs;
        self
    }

    pub fn build(self) -> Result<RouteMessage<H>> {
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
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RouteAttr {
    pub len: u16,
    pub typ: RouteAttrType,
    pub value: Vec<u8>,
}

impl RouteAttr {
    pub fn builder<T: Serialize>() -> RouteAttrBuilder<T> {
        RouteAttrBuilder::new()
    }
}

pub struct RouteAttrBuilder<T> {
    typ: Option<RouteAttrType>,
    value: Option<T>,
}

impl<T: Serialize> RouteAttrBuilder<T> {
    pub fn new() -> Self {
        Self {
            typ: None,
            value: None,
        }
    }

    pub fn typ(mut self, typ: RouteAttrType) -> Self {
        self.typ = Some(typ);
        self
    }

    pub fn value(mut self, value: T) -> Self {
        self.value = Some(value);
        self
    }

    pub fn build(self) -> Result<RouteAttr> {
        let typ = self
            .typ
            .ok_or_else(|| Error::ErrMissingField("type".to_string()))?;
        let value = self
            .value
            .ok_or_else(|| Error::ErrMissingField("value".to_string()))?;
        let value = serialize(&value).map_err(Error::ErrSerialize)?;
        let len = aligned_size_of::<T>() as u16;
        Ok(RouteAttr { typ, len, value })
    }
}

#[cfg(test)]
mod tests {
    use crate::route::{RouteHeader, RouteMessageType, AF_INET};
    use crate::{serialize_aligned, DUMP, REQUEST};

    use super::*;

    #[test]
    fn test_nlmsg_with_no_payload_is_expected_size() {
        let rthdr = RouteHeader::builder().build();
        let bytes = serialize_aligned(rthdr).unwrap();
        assert_eq!(bytes.len(), aligned_size_of::<RouteHeader>());
    }
}

use nix::libc::{RTEXT_FILTER_VF, RTEXT_FILTER_SKIP_STATS};
use crate::bytes::{aligned_size, aligned_size_of, SliceReader};
use crate::route::link::{
    InterfaceInfoMessage, LinkAttrHeader, LinkAttrValue, LinkInfoAttrHeader, LinkAttrType,
    LinkInfoAttrValue,
};
use crate::route::route::RouteMessageType;
use crate::route::AF_INET;
use crate::{Flag, NetlinkMessage, NetlinkStream, Result};
use std::mem::size_of;
use std::net::IpAddr;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Link {
    pub family: u8,
    pub typ: u16,
    pub index: i32,
    pub name: Option<String>,
    pub kind: Option<String>,
    pub addr: Option<IpAddr>,
    pub promiscuity: Option<u32>,
    pub parent_dev_name: Option<String>,
}

#[repr(C)]
#[derive(serde::Serialize)]
pub struct Attr<T: serde::Serialize> {
    header: LinkAttrHeader,
    payload: T,
}

pub type LinkAttr = Attr2<LinkAttrType, LinkAttrValue>;

pub struct Attr2<T, P> {
    len: u16,
    typ: T,
    payload: P,
}

impl NetlinkStream {
    pub fn get_link(&mut self, name: &str) -> Result<Option<Link>> {
        let ifinfomsg = InterfaceInfoMessage::builder()
            .family(0)
            .index(0)
            .build()?;

        // Notes:
        // 1. Attribute length *must* include the size of the `type` and `len`
        //  fields. That is, add 4 bytes to the payload length.
        // 2. The payload length should be the *exact* length of the payload as
        //  bytes, without thinking about alignment
        // 3. The overall byte array (including header then payload) must THEN
        //  be padded to the 4-byte alignment  
        let mut attr1 = {
            let hdr = LinkAttrHeader {
                typ: LinkAttrType::InterfaceName,
                len: (aligned_size_of::<LinkAttrHeader>() + name.as_bytes().len()) as u16
            };

            let mut bytes = bincode::serialize(&hdr).unwrap();
            let mut payload = name.as_bytes().to_vec();
            bytes.append(&mut payload);

            let padding = aligned_size(bytes.len()) - bytes.len();
            bytes.append(&mut vec![0u8; padding]);

            bytes
        };

        let mut attr2 = {
            let hdr = LinkAttrHeader {
                typ: LinkAttrType::ExtMask,
                len: (aligned_size_of::<LinkAttrHeader>() + size_of::<i32>()) as u16
            };
            let val = RTEXT_FILTER_SKIP_STATS | RTEXT_FILTER_VF;

            let mut bytes = bincode::serialize(&hdr).unwrap();
            let mut payload = val.to_le_bytes().to_vec();
            bytes.append(&mut payload);

            let padding = aligned_size(bytes.len()) - bytes.len();
            bytes.append(&mut vec![0u8; padding]);

            bytes
        };

        let nlmsg = NetlinkMessage::builder()
            .typ(RouteMessageType::GetLink)
            .flags(Flag::Request.into())
            .append(ifinfomsg)?
            .append_slice(&mut attr1)?
            .append_slice(&mut attr2)?
            .build();

        self.send(nlmsg)?;

        let msg = match self.recv()? {
            Some(msg) => msg,
            None => return Ok(None),
        };

        let mut reader = SliceReader::new(&msg.payload);
        let ifinfomsg = reader.read::<InterfaceInfoMessage>()?;
        let attrs = read_attributes(&mut reader)?;
        let link = build_link(ifinfomsg, &attrs);

        Ok(Some(link))
    }

    /// List network interfaces.
    ///
    /// # Errors
    ///
    /// Returns  a [`crate::Error`] on failure.
    pub fn list_links(&mut self) -> Result<Vec<Link>> {
        let ifinfomsg = InterfaceInfoMessage::builder().family(AF_INET).build()?;

        let nlmsg = NetlinkMessage::builder()
            .typ(RouteMessageType::GetLink)
            .flags(Flag::Request | Flag::Dump)
            .append(ifinfomsg)?
            .build();

        self.send(nlmsg)?;

        let mut links = vec![];
        while let Ok(Some(msg)) = self.recv() {
            let mut reader = SliceReader::new(&msg.payload);
            let ifinfomsg = reader.read::<InterfaceInfoMessage>()?;
            let attrs = read_attributes(&mut reader)?;
            let link = build_link(ifinfomsg, &attrs);
            links.push(link);
        }

        Ok(links)
    }
}

fn read_attributes(reader: &mut SliceReader) -> Result<Vec<LinkAttrValue>> {
    let mut attributes = vec![];

    while !reader.is_empty() {
        let hdr = reader.read::<LinkAttrHeader>()?;

        let value_len = aligned_size(hdr.len as usize) - aligned_size_of::<LinkAttrHeader>();
        let value_bytes = reader.take(value_len)?;
        let value = LinkAttrValue::deserialize(hdr.typ, value_bytes)?;

        attributes.push(value);
    }

    Ok(attributes)
}

fn build_link(ifinfomsg: InterfaceInfoMessage, attrs: &[LinkAttrValue]) -> Link {
    let mut link = Link {
        family: ifinfomsg.family,
        typ: ifinfomsg.typ,
        index: ifinfomsg.index,
        ..Default::default()
    };

    for attr in attrs {
        match attr {
            LinkAttrValue::InterfaceName(name) => {
                link.name = Some(name.clone());
            }
            LinkAttrValue::Address(addr) => {
                link.addr = Some(*addr);
            }
            LinkAttrValue::Promiscuity(promiscuity) => {
                link.promiscuity = Some(*promiscuity);
            }
            LinkAttrValue::LinkInfo(bytes) => {
                let mut reader = SliceReader::new(bytes);
                let infohdr = reader.read::<LinkInfoAttrHeader>().unwrap();

                let value_len =
                    aligned_size(infohdr.len as usize) - aligned_size_of::<LinkInfoAttrHeader>();
                let value_bytes = reader.take(value_len).unwrap();
                let value = LinkInfoAttrValue::deserialize(infohdr.typ, value_bytes).unwrap();

                match value {
                    LinkInfoAttrValue::Kind(kind) => {
                        link.kind = Some(kind.clone());
                    }
                    _ => {}
                };
            }
            LinkAttrValue::ParentDevName(name) => {
                link.parent_dev_name = Some(name.clone());
            }
            _ => {
                continue;
            }
        }
    }

    link
}
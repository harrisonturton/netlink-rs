use crate::bytes::{aligned_size, aligned_size_of, SliceReader};
use crate::route::link::{InterfaceInfoMessage, LinkAttrHeader, LinkAttrValue};
use crate::route::route::RouteMessageType;
use crate::route::AF_INET;
use crate::{Flag, NetlinkMessage, NetlinkStream, Result};
use std::net::IpAddr;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Link {
    pub family: u8,
    pub typ: u16,
    pub index: i32,
    pub name: Option<String>,
    pub addr: Option<IpAddr>,
    pub promiscuity: Option<u32>,
    pub parent_dev_bus_name: Option<String>,
}

impl NetlinkStream {
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
            log::info!("{link:?}");
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
            LinkAttrValue::ParentDevBusName(name) => {
                link.parent_dev_bus_name = Some(name.clone());
            }
            _ => {
                continue;
            }
        }
    }

    link
}

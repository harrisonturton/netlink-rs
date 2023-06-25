use crate::route::AF_INET;
use crate::route::route::RouteMessageType;
use crate::{NetlinkMessage, Flag, reader::SliceReader, aligned_size, aligned_size_of};
use crate::{NetlinkStream, Result, route::link::{InterfaceInfoMessage}};

use crate::route::route::{RouteAttrValue, RouteAttrHeader};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Link {
    // Empty
}


impl NetlinkStream {
    /// List network interfaces.
    /// 
    /// # Errors
    /// 
    /// Returns  a [`crate::Error`] on failure.
    pub fn list_links(&mut self) -> Result<Vec<Link>> {
        let ifinfomsg = InterfaceInfoMessage::builder()
            .family(AF_INET)
            .build()?;

        let nlmsg = NetlinkMessage::builder()
            .typ(RouteMessageType::GetLink)
            .flags(Flag::Request | Flag::Dump)
            .append(ifinfomsg)?
            .build();

        self.send(nlmsg)?;

        while let Ok(Some(msg)) = self.recv() {
            let mut reader = SliceReader::new(&msg.payload);
            let ifinfomsg = reader.read::<InterfaceInfoMessage>()?;
            log::debug!("{ifinfomsg:?}");

            let attrs = read_attributes(&mut reader)?;
            for attr in attrs {
                log::info!("{attr:?}");
            }
        }

        Ok(Vec::new())
    }
}

fn read_attributes(reader: &mut SliceReader) -> Result<Vec<RouteAttrValue>> {
    let mut attributes = vec![];

    while !reader.is_empty() {
        let hdr = reader.read::<RouteAttrHeader>()?;

        let value_len = aligned_size(hdr.len as usize) - aligned_size_of::<RouteAttrHeader>();
        let value_bytes = reader.take(value_len)?;
        log::debug!("len: {value_len} typ: {:?}", hdr.typ);
        let value = RouteAttrValue::from(hdr.typ, value_bytes)?;
        log::debug!("Succeeded");

        attributes.push(value);
    }

    Ok(attributes)
}

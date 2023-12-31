use super::{RouteAttrHeader, RouteAttrValue, RouteMessage, RouteMessageType};
use crate::bytes::{aligned_size, aligned_size_of, SliceReader};
use crate::route::AF_INET;
use crate::{Flag, NetlinkMessage, NetlinkStream, Result};
use std::net::IpAddr;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Route {
    pub table: u8,
    pub protocol: u8,
    pub scope: u8,
    pub priority: Option<i32>,
    pub gateway: Option<IpAddr>,
    pub dest: Option<IpAddr>,
    pub source: Option<IpAddr>,
    pub preferred_source: Option<IpAddr>,
    pub output_interface_index: Option<i32>,
}

impl NetlinkStream {
    /// List the route table.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on failure.
    pub fn list_routes(&mut self) -> Result<Vec<Route>> {
        let rthdr = RouteMessage::builder().family(AF_INET).build()?;

        let nlmsg = NetlinkMessage::builder()
            .typ(RouteMessageType::GetRoute)
            .flags(Flag::Request | Flag::Dump)
            .append(rthdr)?
            .build();

        self.send(nlmsg)?;

        let mut routes = vec![];
        while let Ok(Some(msg)) = self.recv() {
            let (rtmsg, rattrs) = read_rtmsg(&msg)?;
            let route = build_route(&rtmsg, &rattrs);
            routes.push(route);
        }

        Ok(routes)
    }
}

fn read_rtmsg(msg: &NetlinkMessage) -> Result<(RouteMessage, Vec<RouteAttrValue>)> {
    let mut reader = SliceReader::new(&msg.payload);
    let rtmsg = reader.read::<RouteMessage>()?;

    let mut attributes = vec![];

    while !reader.is_empty() {
        let hdr = reader.read::<RouteAttrHeader>()?;

        let value_len = aligned_size(hdr.len as usize) - aligned_size_of::<RouteAttrHeader>();
        let value_bytes = reader.take(value_len)?;
        let value = RouteAttrValue::deserialize(hdr.typ, value_bytes)?;

        attributes.push(value);
    }

    Ok((rtmsg, attributes))
}

fn build_route(msg: &RouteMessage, attrs: &[RouteAttrValue]) -> Route {
    let mut route = Route {
        table: msg.table,
        protocol: msg.protocol,
        scope: msg.scope,
        ..Default::default()
    };

    for attr in attrs {
        match attr {
            RouteAttrValue::Dest(addr) => {
                route.dest = Some(*addr);
            }
            RouteAttrValue::PreferredSourceAddr(addr) => {
                route.preferred_source = Some(*addr);
            }
            RouteAttrValue::OutputInterfaceIndex(index) => {
                route.output_interface_index = Some(*index);
            }
            RouteAttrValue::Priority(priority) => {
                route.priority = Some(*priority);
            }
            RouteAttrValue::Gateway(addr) => {
                route.gateway = Some(*addr);
            }
            RouteAttrValue::Table(_) => {
                continue;
            }
            _ => {
                log::warn!("received unexpected route attribute: {attr:?}");
            }
        }
    }

    route
}

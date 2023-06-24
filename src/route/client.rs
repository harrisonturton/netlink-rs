use std::net::IpAddr;
use crate::aligned_size;
use crate::aligned_size_of;
use crate::reader::SliceReader;
use crate::route::RouteAttrHeader;
use crate::route::RouteMessage;
use crate::route::RouteMessageType;
use crate::route::AF_INET;
use crate::Flag;
use crate::NetlinkMessage;
use crate::NetlinkStream;
use crate::Result;

use super::RouteAttrValue;

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
            let route = build_route(&rtmsg, &rattrs)?;
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
        let value_bytes = reader.take(value_len as usize)?;
        let value = RouteAttrValue::from(hdr.typ, value_bytes)?;

        attributes.push(value);
    }

    Ok((rtmsg, attributes))
}

fn build_route(msg: &RouteMessage, attrs: &[RouteAttrValue]) -> Result<Route> {
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

    Ok(route)
}
use crate::{NetlinkSocket, Result, NetlinkMessage, REQUEST, DUMP, NetlinkMessageStream};
use super::{RouteHeader, RouteMessage};

impl NetlinkSocket {
    pub fn dump_route_table(&mut self) -> Result<NetlinkMessageStream> {
        let rthdr = RouteHeader::builder()
            .family(super::AF_INET)
            .build();

        let rtmsg = RouteMessage::builder()
            .header(rthdr)
            .attrs(vec![])
            .build()?;

        let nlmsg = NetlinkMessage::builder()
            .payload(&rtmsg)
            .typ(super::GET_ROUTE)
            .flags(REQUEST | DUMP)
            .build()?;

        self.send(nlmsg)?;
        self.recv()
    }
}
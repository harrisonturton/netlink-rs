// use super::{RouteHeader, RouteMessage};
// use crate::route::{InterfaceAddrHeader, RouteAttr, RouteAttrType, RouteMessageType, AF_INET};
// use crate::{
//     Error, NetlinkMessage, NetlinkMessageStream, NetlinkSocket, Result, CREATE, DUMP, EXCL, REQUEST,
// };
// use nix::net::if_::if_nametoindex;
// use std::net::IpAddr;

// impl NetlinkSocket {
//     pub fn dump_route_table(&mut self) -> Result<NetlinkMessageStream> {
//         let rthdr = RouteHeader::builder().family(super::AF_INET).build();

//         let rtmsg = RouteMessage::builder()
//             .header(rthdr)
//             .attrs(vec![])
//             .build()?;

//         let nlmsg = NetlinkMessage::builder()
//             .append(&rtmsg)?
//             .typ(RouteMessageType::GetRoute)
//             .flags(REQUEST | DUMP)
//             .build();

//         self.send(nlmsg)?;
//         self.recv()
//     }

//     // pub fn new_link(
//     //     &mut self,
//     //     iface_name: &str,
//     //     addr: &IpAddr,
//     //     prefixlen: u8,
//     // ) -> Result<NetlinkMessageStream> {
//     //     let index = if_nametoindex(iface_name).map_err(Error::ErrNameToIndex)?;

//     //     let ifinfomsg = InterfaceInfoHeader::builder()
//     //         .family(AF_INET)
//     //         .build();

//     //     let rattr = RouteAttr::builder()
//     //         .typ(RouteAttrType::IfaAddress)
//     //         .value(addr)
//     //         .build()?;

//     //     let rtmsg = RouteMessage::builder()
//     //         .header(ifaddrmsg)
//     //         .attrs(vec![rattr])
//     //         .build()?;

//     //     let nlmsg = NetlinkMessage::builder()
//     //         .payload(&rtmsg)
//     //         .typ(RouteMessageType::NewAddr)
//     //         .flags(REQUEST | CREATE | EXCL)
//     //         .build()?;

//     //     self.send(nlmsg)?;
//     //     self.recv()
//     // }

//     pub fn new_addr(
//         &mut self,
//         iface_name: &str,
//         addr: &IpAddr,
//         prefixlen: u8,
//     ) -> Result<NetlinkMessageStream> {
//         let index = if_nametoindex(iface_name).map_err(Error::ErrNameToIndex)?;

//         let ifaddrmsg = InterfaceAddrHeader::builder()
//             .family(AF_INET)
//             .index(index)
//             .prefixlen(prefixlen)
//             .build();

//         let rattr = RouteAttr::builder()
//             .typ(RouteAttrType::IfaLocal)
//             .value(addr)
//             .build()?;

//         let rtmsg = RouteMessage::builder()
//             .header(ifaddrmsg)
//             .attrs(vec![rattr])
//             .build()?;

//         let nlmsg = NetlinkMessage::builder()
//             .append(&rtmsg)?
//             .typ(RouteMessageType::NewAddr)
//             .flags(REQUEST | CREATE | EXCL)
//             .build();

//         self.send(nlmsg)?;
//         self.recv()
//     }
// }

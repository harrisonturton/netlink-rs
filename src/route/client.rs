use super::flags::GET_ROUTE;
use super::{RouteHeader, MessageType};
use crate::flags::{DUMP, REQUEST};
use crate::{NetlinkMessage, Result};

// impl Conn {
//     pub fn get_route(&mut self) -> Result<Message<Vec<u8>>> {
//         let req = Message::builder()
//             .payload(&Header {
//                 family: MessageType::GETROUTE as u8,
//                 ..Default::default()
//             })
//             .typ(GET_ROUTE as u16)
//             .flags(REQUEST | DUMP)
//             .build()?;

//         self.send(&req)?;

//         let mut reader = self.recv()?;
//         let mut messages = vec![];
//         loop {
//             let msg = reader.try_next()?;
//             if msg.has_type(crate::MessageType::Done) {
//                 break;
//             }
//             messages.push(msg);
//         }

//         todo!()
//     }
// }

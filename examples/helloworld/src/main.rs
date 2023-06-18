use netlink::{
    flags::{DUMP, REQUEST},
    Message,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    let sock = netlink::connect()?;
    let message = dump_routes_req();
    sock.send(&message)?;

    let mut reader = sock.recv()?;
    while let Ok(msg) = reader.try_next() {
        println!("{msg:?}");
        if msg.header.typ == 3 {
            break;
        }
    }

    Ok(())
}

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct RouteMessage {
    pub rtm_family: u8,
    pub rtm_dst_len: u8,
    pub rtm_src_len: u8,
    pub rtm_tos: u8,
    pub rtm_table: u8,
    pub rtm_protocol: u8,
    pub rtm_scope: u8,
    pub rtm_type: u8,
    pub rtm_flags: u8,
}

pub fn dump_routes_req() -> Message<Vec<u8>> {
    let payload = RouteMessage {
        rtm_family: 2, // AF_INIT
        ..Default::default()
    };
    Message::builder()
        .payload(&payload)
        .typ(0x1A) // RTM_GETROUTE
        .flags(REQUEST | DUMP)
        .build()
        .unwrap()
}

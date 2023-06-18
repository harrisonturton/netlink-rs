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

pub fn dump_routes_req() -> Message<Vec<u8>> {
    let payload = netlink::route::Header {
        family: 2, // AF_INIT
        ..Default::default()
    };
    Message::builder()
        .payload(&payload)
        .typ(0x1A) // RTM_GETROUTE
        .flags(REQUEST | DUMP)
        .build()
        .unwrap()
}

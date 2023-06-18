use std::error::Error;
use netlink::{route, flags, Message};

pub const AF_INET: u8 = 0x2;
pub const RTM_GETROUTE: u16 = 0x1A;

fn main() -> Result<(), Box<dyn Error>> {
    let dump_routes_req = Message::builder()
        .payload(&route::Header {
            family: AF_INET,
            ..Default::default()
        })
        .typ(RTM_GETROUTE)
        .flags(flags::REQUEST | flags::DUMP)
        .build()?;

    let sock = netlink::connect()?;
    sock.send(&dump_routes_req)?;

    let mut reader = sock.recv()?;
    while let Ok(msg) = reader.try_next() {
        println!("{msg:?}");

        if msg.header.typ == netlink::MessageType::Done as u16 {
            break;
        }
    }

    Ok(())
}
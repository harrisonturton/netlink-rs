use std::error::Error;
use netlink::route::{RouteHeader, RouteMessageType, AF_INET};
use netlink::{NetlinkStream, Flag, NetlinkMessage};

fn main() -> Result<(), Box<dyn Error>> {
    let mut conn = NetlinkStream::connect()?;

    let rthdr = RouteHeader::builder()
        .family(AF_INET)
        .build();

    let msg = NetlinkMessage::builder()
        .typ(RouteMessageType::GetRoute)
        .flags(Flag::Request | Flag::Dump)
        .append(rthdr)?
        .build();

    conn.send(msg)?;

    for msg in conn.into_iter(){
        println!("{msg:?}");
    }

    Ok(())
}
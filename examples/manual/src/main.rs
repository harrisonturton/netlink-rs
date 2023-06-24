use netlink::route::{RouteMessage, RouteMessageType, AF_INET};
use netlink::{Flag, NetlinkMessage, NetlinkStream};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    logger::init().unwrap();

    let mut conn = NetlinkStream::connect()?;

    // This example uses types already declared in the library, but you could
    // write your own. It just needs to implement `serde::Serialize`.
    let rthdr = RouteMessage::builder().family(AF_INET).build()?;

    let msg = NetlinkMessage::builder()
        .typ(RouteMessageType::GetRoute)
        .flags(Flag::Request | Flag::Dump)
        .append(rthdr)?
        .build();

    conn.send(msg)?;

    while let Ok(Some(msg)) = conn.recv() {
        log::info!(
            "Received a Netlink message with a {} byte payload",
            msg.payload.len()
        );
    }

    Ok(())
}

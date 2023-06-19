use netlink::route::{self, RouteHeader, RouteMessage};
use netlink::{NetlinkMessage, NetlinkSocket, DUMP, REQUEST};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Build netlink route payload
    let rthdr = RouteHeader::builder()
        .family(route::AF_INET)
        .build();

    let rtmsg = RouteMessage::builder()
        .header(rthdr)
        .attrs(vec![])
        .build()?;

    // Wrap in netlink message
    let nlmsg = NetlinkMessage::builder()
        .payload(&rtmsg)
        .typ(route::GET_ROUTE)
        .flags(REQUEST | DUMP)
        .build()?;

    // Send request
    let mut sock = NetlinkSocket::connect()?;
    sock.send(nlmsg)?;

    // Read multipart response
    for message in sock.recv()?.messages() {
        println!("{message:?}");
    }

    Ok(())
}
use netlink::flags::{DUMP, REQUEST};
use netlink::route::{RouteHeader, RouteMessage};
use netlink::{NetlinkMessage, NetlinkSocket};
use std::error::Error;

// Netlink message type for fetching route data
pub const RTM_GETROUTE: u16 = 0x1A;

// Socket family type used by rnetlink
pub const AF_INET: u8 = 0x2;

fn main() -> Result<(), Box<dyn Error>> {
    // Build netlink route payload
    let rthdr = RouteHeader::builder()
        .family(AF_INET)
        .build();

    let rtmsg = RouteMessage::builder()
        .header(rthdr)
        .attrs(vec![])
        .build()?;

    // Wrap in netlink message
    let nlmsg = NetlinkMessage::builder()
        .payload(&rtmsg)
        .typ(RTM_GETROUTE)
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
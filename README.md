## netlink-rs

A library for interacting
[Netlink-based](https://man7.org/linux/man-pages/man7/netlink.7.html) Linux
kernel interfaces from Rust.

This only implements the core Netlink protocol. It could be described as
implementing the transport layer of higher-level interfaces like
[`rnetlink`](https://man7.org/linux/man-pages/man7/rtnetlink.7.html).

## Usage

The following snippet will dump the kernel's main routing table. Note that the
`rnetlink` header must be wrapped in the `netlink` header. This example
demonstrates how the library implements the netlink transport, but not the
protocol extensions built upon it.

```rust
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
```
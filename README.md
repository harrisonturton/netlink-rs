## netlink-rs

A library for interacting
[Netlink-based](https://man7.org/linux/man-pages/man7/netlink.7.html) Linux
kernel interfaces from Rust. This abstracts over the core protocol.  It should
be viewed as an implementation of the Netlink "transport" layer, rather than an
abstraction over the subsystem-specific protocols.

The [`NETLINK_ROUTE`](https://man7.org/linux/man-pages/man7/rtnetlink.7.html)
family is current in development. It is partially supported, and used in the
example below.

## Installation

#### With all features

Add the following line to your `Cargo.toml`:

```rust
netlink-rs = { git = "https://github.com/harrisonturton/netlink-rs/tree/main.git", features = ["all"]}
```

#### Core Netlink only

This will install only the `core` subpackage:

```rust
netlink-rs = { git = "https://github.com/harrisonturton/netlink-rs/tree/main.git" }
```

#### Netlink route only

This will install only the `route` subpackage:

```rust
netlink-rs = { git = "https://github.com/harrisonturton/netlink-rs/tree/main.git", features = ["route"] }
```

## Usage

The following snippet will dump the kernel's main routing table. This
demonstrates how a subsystem-specific message, `RouteMessage`, must be wrapped
in the `NetlinkMessage` to send to the kernel.

Note how the specifics of the Netlink socket protocol are hidden. You only need
to implement the payload types from each subsystem-specific protocol in order to
call netlink interfaces.

```rust
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
```

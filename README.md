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

Add the following line to your `Cargo.toml`:

```rust
netlink-rs = { git = "https://github.com/harrisonturton/netlink-rs/tree/main.git" }
```

## Usage

The following examples will all dump the kernel's routing tables, in order of
highest to lowest abstraction.

### Convenience methods

This crate has the best support for `rnetlink`. Depending on what information
you need, it might be exposed by one of these easy-to-use wrapper methods. They
can be called directly and gather the responses into user-friendly structs.

```rust
use netlink::NetlinkStream;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut conn = NetlinkStream::connect()?;

    for route in conn.list_routes()? {
        println!("{route:?}");
    }

    Ok(())
}
```

### Using Netlink directly

But if this crate doesn't support the protocol or method you need (there's a lot
of Netlink interfaces!) it has enough escape hatches for you to implement it
manually.

This example shows how a `NetlinkMessage` can be built and passed to the
`NetlinkStream`. Once this request message has been sent, one or more (in the
case of multipart messages) responses can be read.

Note how the specifics of the Netlink socket protocol are hidden. You only need
to implement the payload types from each subsystem-specific protocol in order to
call netlink interfaces.

```rust
use std::error::Error;
use netlink::route::{RouteHeader, RouteMessageType, AF_INET};
use netlink::{NetlinkStream, Flag, NetlinkMessage};

fn main() -> Result<(), Box<dyn Error>> {
    let mut conn = NetlinkStream::connect()?;

    // This example uses types already declared in the library, but you could
    // write your own. It just needs to implement `serde::Serialize`.
    let rthdr = RouteHeader::builder()
        .family(AF_INET)
        .build();

    let msg = NetlinkMessage::builder()
        .typ(RouteMessageType::GetRoute)
        .flags(Flag::Request | Flag::Dump)
        .append(rthdr)?
        .build();

    conn.send(msg)?;

    while let Ok(Some(msg)) = conn.recv() {
        println!("{msg:?}");
    }

    Ok(())
}
```

## Contributing

Please do! There are many Netlink interfaces; I don't have time to implement all
of them. If appreciate it if any extensions are submitted upstream.
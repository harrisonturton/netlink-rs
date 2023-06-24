use netlink::NetlinkStream;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut conn = NetlinkStream::connect()?;

    for route in conn.list_routes()? {
        println!("{route:?}");
    }

    Ok(())
}
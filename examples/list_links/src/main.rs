use netlink::NetlinkStream;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    logger::init().unwrap();

    let mut conn = NetlinkStream::connect()?;

    for link in conn.list_links()? {
        log::info!(
            "index={:?} name={:?} addr={:?}",
            link.index,
            link.name,
            link.addr,
        );
    }

    Ok(())
}

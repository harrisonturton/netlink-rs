use netlink::NetlinkStream;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    logger::init().unwrap();

    let mut conn = NetlinkStream::connect()?;

    let link = conn.get_link("lo")?;
    log::info!("{link:?}");

    // for link in conn.list_links()? {
    //     log::info!("{link:?}");
    //     // log::info!(
    //     //     "index={:?} name={:?} addr={:?}",
    //     //     link.index,
    //     //     link.name,
    //     //     link.addr,
    //     // );
    // }

    Ok(())
}
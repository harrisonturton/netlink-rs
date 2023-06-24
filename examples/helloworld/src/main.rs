use netlink::NetlinkStream;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    logger::init().unwrap();

    let mut conn = NetlinkStream::connect()?;

    for route in conn.list_routes()? {
        log::info!(
            "table={:?} gateway={:?} dest={:?} source={:?} prefsrc={:?}",
            route.table,
            route.gateway,
            route.dest,
            route.source,
            route.preferred_source
        );
    }

    Ok(())
}

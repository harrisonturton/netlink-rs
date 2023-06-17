mod withnix;
mod withnix2;
// mod withlibc;

fn main() {
    logger::init().unwrap();
    log::info!("Starting up");
    withnix2::main().unwrap();
}

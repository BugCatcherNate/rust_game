mod core;
use log::{debug};
use pollster;

fn main() {
    env_logger::init();
    debug!("game start");
    pollster::block_on(core::state::run());
}


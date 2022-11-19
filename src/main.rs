mod core;
use log::{debug};

fn main() {
    env_logger::init();
    debug!("game start");
    core::main_window::run();
}


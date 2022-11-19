mod core;
use log::{debug};

fn main() {
    env_logger::init();
    debug!("Application started");
    core::main_window::run();
}


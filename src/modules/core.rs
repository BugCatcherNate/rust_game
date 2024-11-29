use log::{debug};
use super::window;

pub fn initialize(){
    debug!("Application initialize");
    window::run_window();

}

mod modules;

fn main() {
    env_logger::init();
    modules::core::initialize();
}

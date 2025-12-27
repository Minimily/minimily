use env_logger::Env;
use minimily::config::load_config;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = load_config().expect("Failed to load config");
    log::info!("Port: {}", config.server_port);
}
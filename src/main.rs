use minimily::config::load_config;

fn main() {
    let config = load_config().expect("Failed to load config");
    println!("Port: {}", config.server_port);
}
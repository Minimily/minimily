use dotenv::dotenv;
use std::env;
use std::io::Error;

#[derive(serde::Deserialize)]
pub struct Config {
    pub server_port: u16,
}

pub fn load_config() -> Result<Config, Error> {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().expect("Configuration: PORT must be a number");

    let config: Config = Config {
        server_port: port,
    };

    Ok(config)
}
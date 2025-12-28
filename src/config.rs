use dotenv::dotenv;
use std::env;
use std::io::Error;

#[derive(serde::Deserialize)]
pub struct Config {
    pub server_port: u16,
    pub database: DatabaseConfig,
}

#[derive(serde::Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

pub fn load_config() -> Result<Config, Error> {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().expect("Configuration: PORT must be a number");
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "".to_string());

    let config: Config = Config {
        server_port: port,
        database: DatabaseConfig { url: database_url },
    };

    Ok(config)
}
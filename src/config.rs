use dotenv::dotenv;
use std::env;
use std::io::Error;

#[derive(serde::Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server_port: u16,
    pub session: SessionConfig,
}

#[derive(serde::Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(serde::Deserialize)]
pub struct SessionConfig {
    pub secrete_key: String,
    pub secure: bool,
}

pub fn load_config() -> Result<Config, Error> {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().expect("Configuration: PORT must be a number");
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "".to_string());
    let session_secrete_key = env::var("SESSION_SECRET_KEY").unwrap_or_else(|_| "".to_string());
    let session_secure = env::var("SESSION_SECURE").unwrap_or_else(|_| "false".to_string()).parse().unwrap_or(false);

    let config: Config = Config {
        database: DatabaseConfig { url: database_url },
        server_port: port,
        session: SessionConfig {
            secrete_key: session_secrete_key,
            secure: session_secure,
        }
    };

    Ok(config)
}
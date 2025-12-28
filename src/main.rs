use env_logger::Env;
use sqlx::PgPool;
use minimily::config::{load_config, Config};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = load_config().expect("Failed to load config");
    let _pool = match get_pool(&config).await {
        Ok(pool) => pool,
        Err(e) => {
            log::error!("Error connecting to database: {}", e);
            return;
        }
    };

    log::info!("Port: {}", config.server_port);
}

async fn get_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    match PgPool::connect(&config.database.url).await {
        Ok(pool) => Ok(pool),
        Err(e) => Err(e)
    }
}
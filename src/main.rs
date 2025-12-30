use env_logger::Env;
use sqlx::{migrate, PgPool};
use minimily::config::{load_config, Config};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = load_config().expect("Failed to load config");

    let pool = match get_pool(&config).await {
        Ok(pool) => {
            log::info!("Connected to the database");
            pool
        },
        Err(e) => {
            log::error!("Error connecting to database: {}", e);
            return;
        }
    };

    migrate_database(&pool).await;

    log::info!("Port: {}", config.server_port);
}

async fn get_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    match PgPool::connect(&config.database.url).await {
        Ok(pool) => Ok(pool),
        Err(e) => Err(e)
    }
}

async fn migrate_database(pool: &PgPool) {
    migrate!("storage/migrations")
        .run(pool)
        .await
        .expect("Failed to migrate database");
    log::info!("Database migrated");
}
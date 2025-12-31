use sqlx::PgPool;
use tera::Tera;

pub struct AppState {
    pub pool: PgPool,
    pub template: Tera,
}

impl AppState {
    pub fn new(pool: PgPool, template: Tera) -> Self {
        AppState { pool, template }
    }
}
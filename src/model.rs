use chrono::{NaiveDate, NaiveDateTime};
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UserAccount {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<NaiveDate>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub created: Option<NaiveDateTime>,
    pub modified: Option<NaiveDateTime>,
}

impl UserAccount {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
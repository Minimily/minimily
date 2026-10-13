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

#[derive(serde::Serialize, Clone)]
pub struct UserAccount {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<NaiveDate>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: Option<String>,
    pub created: Option<NaiveDateTime>,
    pub modified: Option<NaiveDateTime>,
}

impl UserAccount {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[derive(serde::Serialize, Clone)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub group_type: GroupType,
    pub created_by: i32,
    pub created: Option<NaiveDateTime>,
}

#[derive(serde::Serialize, Clone, Debug, PartialEq, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum GroupType {
    Family,
}

#[derive(serde::Serialize, Clone)]
pub struct GroupMember {
    pub id: i32,
    pub group_id: i32,
    pub user_account: UserAccount,
    pub joined: Option<NaiveDateTime>,
}

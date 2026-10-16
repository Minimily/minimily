use chrono::{NaiveDate, NaiveDateTime};
use sqlx::{PgPool};
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
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: Option<String>,
    pub created: Option<NaiveDateTime>,
    pub modified: Option<NaiveDateTime>,
}

impl UserAccount {
    pub fn full_name(&self) -> String {
        match &self.first_name {
            Some(first) => {
                match &self.last_name {
                    Some(last) => {
                        return format!("{} {}", first, last)
                    }
                    None => return format!("{}", first)
                }
            }
            None => {
                match &self.last_name {
                    Some(last) => {
                        return format!("{}", last)
                    }
                    None => return "".to_string()
                }
            }
        }
    }

    pub fn new_user_account(id: i32) -> Self {
        UserAccount {
            id,
            first_name: None,
            last_name: None,
            birth_date: None,
            email: None,
            phone: None,
            password: None,
            created: None,
            modified: None,
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct UserGroup {
    pub id: i32,
    pub name: Option<String>,
    pub group_type: Option<GroupType>,
    pub created_by: Option<UserAccount>,
    pub created_date: Option<NaiveDateTime>,
}

impl UserGroup {
    pub fn new_user_group(id: i32) -> Self {
        UserGroup {
            id,
            name: None,
            group_type: None,
            created_by: None,
            created_date: None,
        }
    }
}

#[derive(serde::Serialize, Clone, Debug, PartialEq, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum GroupType {
    Family,
}

#[derive(serde::Serialize, Clone)]
pub struct GroupMember {
    pub id: i32,
    pub user_group: Option<UserGroup>,
    pub user_account: Option<UserAccount>,
    pub created_by: Option<UserAccount>,
    pub created_date: Option<NaiveDateTime>,
}

#[derive(serde::Serialize, Clone)]
pub struct Good {
    pub id: i32,
    pub user_group: Option<UserGroup>,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<UserAccount>,
    pub created_date: Option<NaiveDateTime>,
}

use chrono::{NaiveDateTime};
use crate::account::model::{UserAccount, UserGroup};

#[derive(serde::Serialize, Clone)]
pub struct Good {
    pub id: i32,
    pub user_group: Option<UserGroup>,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<GoodLocation>,
    pub created_by: Option<UserAccount>,
    pub created_date: Option<NaiveDateTime>,
}

#[derive(serde::Serialize, Clone)]
pub struct GoodLocation {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<UserAccount>,
    pub created_date: Option<NaiveDateTime>,
}

impl GoodLocation {
    pub fn new_good_location(id: i32) -> Self {
        GoodLocation {
            id,
            name: None,
            description: None,
            created_by: None,
            created_date: None,
        }
    }
}

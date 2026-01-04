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
    pub password: Option<String>,
    pub created: Option<NaiveDateTime>,
    pub modified: Option<NaiveDateTime>,
}

impl UserAccount {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
    
    pub fn new_profile(&self) -> Profile {
        Profile {
            id: 0,
            name: self.full_name(),
            profile_type: ProfileType::UserAccount,
            user_account: Some(self.clone()),
            created_by: self.clone(),
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct Profile {
    pub id: i32,
    pub name: String,
    pub profile_type: ProfileType,
    pub user_account: Option<UserAccount>,
    pub created_by: UserAccount,
}

#[derive(serde::Serialize, Clone, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum ProfileType {
    UserAccount,
    Family,
}

#[derive(serde::Serialize, Clone)]
pub struct Relationship {
    pub id: i32,
    pub profile_from: Profile,
    pub profile_to: Profile,
    pub relationship_type: RelationshipType,
}

#[derive(serde::Serialize, Clone, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum RelationshipType {
    FamilyMember,
    UserAunt,
    UserDaughter,
    UserBrother,
    UserBrotherInLaw,
    UserCousin,
    UserFather,
    UserFriend,
    UserHusband,
    UserMother,
    UserSister,
    UserSisterInLaw,
    UserSon,
    UserUncle,
    UserWife,
}
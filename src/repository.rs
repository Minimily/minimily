use sqlx::{Error, PgConnection, Row};
use sqlx::postgres::{PgRow, PgPool};
use crate::model::{Profile, Relationship, UserAccount};

pub async fn get_user_account_by_email(conn: &PgPool, email: String) -> Result<UserAccount, Error> {
    sqlx::query("
        select *
        from user_account
        where email = $1
        ")
        .bind(email)
        .map(|row: PgRow| UserAccount {
            id: row.get("id"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            birth_date: row.get("birth_date"),
            email: row.get("email"),
            password: row.get("password"),
            created: row.get("created"),
            modified: row.get("modified"),
        })
        .fetch_one(conn)
        .await
}

pub async fn create_user_account(conn: &mut PgConnection, user_account: UserAccount) -> Result<UserAccount, Error> {
    let user_account = sqlx::query("
        insert into user_account (first_name, last_name, birth_date, email, password)
        values ($1, $2, $3, $4, $5)
        returning id, created, modified
    ")
        .bind(user_account.first_name.clone())
        .bind(user_account.last_name.clone())
        .bind(user_account.birth_date.clone())
        .bind(user_account.email.clone())
        .bind(user_account.password.clone())
        .map(|row: PgRow| UserAccount {
            id: row.get("id"),
            first_name: user_account.first_name.clone(),
            last_name: user_account.last_name.clone(),
            birth_date: user_account.birth_date,
            email: user_account.email.clone(),
            password: user_account.password.clone(),
            created: row.get("created"),
            modified: row.get("modified"),
        }).fetch_one(&mut *conn).await;

    match user_account {
        Ok(ua) => {
            let profile = ua.new_profile();
            create_profile(conn, profile).await?;
            Ok(ua)
        },
        Err(e) => {
            Err(e)
        }
    }
}

pub async fn get_profile(conn: &PgPool, email: String) -> Result<Profile, Error> {
    sqlx::query("
        select ua.id as user_id, ua.first_name, ua.last_name, ua.birth_date, ua.email,
               p.id as profile_id, p.name, p.type
        from user_account ua
            join profile p on p.user_account = ua.id
        where ua.email = $1
    ")
        .bind(email)
        .map(|row: PgRow| Profile {
            id: row.get("profile_id"),
            name: row.get("name"),
            profile_type: row.get("type"),
            user_account: Some(UserAccount {
                id: row.get("user_id"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                birth_date: row.get("birth_date"),
                email: row.get("email"),
                password: None,
                created: None,
                modified: None,
            }),
            created_by: None,
        })
        .fetch_one(conn)
        .await
}

pub async fn get_family_profile(conn: &PgPool, profile: Profile) -> Result<Profile, Error> {
    sqlx::query("
        select p.*
        from relationship r
	        join profile p on p.id = r.profile_to 
        where r.profile_from  = $1
	        and r.type = 'familymember'
        ")
        .bind(profile.id)
        .map(|row: PgRow| Profile {
            id: row.get("id"),
            name: row.get("name"),
            profile_type: row.get("type"),
            user_account: None,
            created_by: None,
        })
        .fetch_one(conn)
        .await
}

pub async fn create_profile(conn: &mut PgConnection, profile: Profile) -> Result<Profile, Error> {
    sqlx::query("
        insert into profile (name, type, user_account, created_by)
        values ($1, $2, $3, $4)
        returning id
    ")
        .bind(profile.name.clone())
        .bind(profile.profile_type.clone())
        .bind(match &profile.user_account {
            Some(user_account) => Some(user_account.id),
            None => None
        })
        .bind(match &profile.created_by {
            Some(user_account) => Some(user_account.id),
            None => None
        })
        .map(|row: PgRow| Profile {
            id: row.get("id"),
            name: profile.name.clone(),
            profile_type: profile.profile_type.clone(),
            user_account: profile.user_account.clone(),
            created_by: profile.created_by.clone(),
        }).fetch_one(conn).await
}

pub async fn create_relationship(conn: &PgPool, relationship: Relationship) -> Result<Relationship, Error> {
    sqlx::query("
        insert into relationship (profile_from, profile_to, type)
        values ($1, $2, $3)
        returning id
    ")
        .bind(relationship.profile_from.id)
        .bind(relationship.profile_to.id)
        .bind(relationship.relationship_type.clone())
        .map(|row: PgRow| Relationship {
            id: row.get("id"),
            profile_from: relationship.profile_from.clone(),
            profile_to: relationship.profile_to.clone(),
            relationship_type: relationship.relationship_type.clone(),
        }).fetch_one(conn).await
}
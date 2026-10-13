use chrono::NaiveDate;
use sqlx::{Error, PgConnection, Row};
use sqlx::postgres::{PgRow, PgPool};
use crate::model::{Group, GroupMember, GroupType, UserAccount};

pub async fn num_user_accounts(conn: &PgPool) -> Result<i64, Error> {
    sqlx::query("select count(id) num from user_account")
        .map(|row: PgRow| row.get("num"))
        .fetch_one(conn)
        .await
}

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
            phone: row.get("phone"),
            password: row.get("password"),
            created: row.get("created"),
            modified: row.get("modified"),
        })
        .fetch_one(conn)
        .await
}

pub async fn create_user_account(conn: &mut PgConnection, user_account: UserAccount) -> Result<UserAccount, Error> {
    sqlx::query("
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
            phone: user_account.phone.clone(),
            password: user_account.password.clone(),
            created: row.get("created"),
            modified: row.get("modified"),
        }).fetch_one(conn).await
}

fn row_to_group(row: &PgRow) -> Group {
    Group {
        id: row.get("id"),
        name: row.get("name"),
        group_type: row.get("type"),
        created_by: row.get("created_by"),
        created: row.get("created"),
    }
}

pub async fn get_user_group(conn: &PgPool, user_account_id: i32, group_type: GroupType) -> Result<Group, Error> {
    sqlx::query(r#"
        select g.*
        from "group" g
            join group_member gm on gm.group_id = g.id
        where gm.user_account = $1
            and g.type = $2
        order by g.id
        limit 1
        "#)
        .bind(user_account_id)
        .bind(group_type)
        .map(|row: PgRow| row_to_group(&row))
        .fetch_one(conn)
        .await
}

pub async fn get_group_members(conn: &PgPool, group_id: i32) -> Result<Vec<GroupMember>, Error> {
    sqlx::query("
        select gm.id as member_id, gm.group_id, gm.joined,
               ua.id as user_id, ua.first_name, ua.last_name, ua.birth_date, ua.email, ua.phone, ua.created
        from group_member gm
            join user_account ua on ua.id = gm.user_account
        where gm.group_id = $1
        order by ua.first_name, ua.last_name
        ")
        .bind(group_id)
        .map(|row: PgRow| GroupMember {
            id: row.get("member_id"),
            group_id: row.get("group_id"),
            user_account: UserAccount {
                id: row.get("user_id"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                birth_date: row.get("birth_date"),
                email: row.get("email"),
                phone: row.get("phone"),
                password: None,
                created: row.get("created"),
                modified: None,
            },
            joined: row.get("joined"),
        })
        .fetch_all(conn)
        .await
}

pub async fn create_group(conn: &PgPool, name: &str, group_type: GroupType, creator_id: i32) -> Result<Group, Error> {
    let mut tx = conn.begin().await?;

    let group = sqlx::query(r#"
        insert into "group" (name, type, created_by)
        values ($1, $2, $3)
        returning *
        "#)
        .bind(name)
        .bind(group_type)
        .bind(creator_id)
        .map(|row: PgRow| row_to_group(&row))
        .fetch_one(&mut *tx)
        .await?;

    add_group_member(&mut tx, group.id, creator_id).await?;

    tx.commit().await?;
    Ok(group)
}

pub async fn add_group_member(conn: &mut PgConnection, group_id: i32, user_account_id: i32) -> Result<(), Error> {
    sqlx::query("insert into group_member (group_id, user_account) values ($1, $2)")
        .bind(group_id)
        .bind(user_account_id)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn update_user_account(conn: &PgPool, id: i32, first_name: &str, last_name: &str, birth_date: Option<NaiveDate>, email: &str, phone: Option<&str>) -> Result<(), Error> {
    sqlx::query("
        update user_account
        set first_name = $1, last_name = $2, birth_date = $3, email = $4, phone = $5, modified = now()
        where id = $6
    ")
        .bind(first_name)
        .bind(last_name)
        .bind(birth_date)
        .bind(email)
        .bind(phone)
        .bind(id)
        .execute(conn)
        .await?;

    Ok(())
}

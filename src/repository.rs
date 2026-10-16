use sqlx::{Error, PgConnection, Row};
use sqlx::postgres::{PgRow, PgPool};
use crate::model::{Good, UserGroup, GroupMember, GroupType, UserAccount};

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

fn row_to_group(row: &PgRow) -> UserGroup {
    UserGroup {
        id: row.get("id"),
        name: row.get("name"),
        group_type: row.get("type"),
        created_by: Some(UserAccount::new_user_account(row.get("created_by"))),
        created_date: row.get("created_date"),
    }
}

pub async fn get_user_group(conn: &PgPool, user_account_id: i32, group_type: GroupType) -> Result<UserGroup, Error> {
    sqlx::query(r#"
        select ug.*
        from user_group ug
            join group_member gm on gm.user_group = ug.id
        where gm.user_account = $1
            and ug.type = $2
        order by ug.id
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
        select gm.id as member_id, gm.user_group, gm.created_date, gm.created_by,
               ua.id as user_id, ua.first_name, ua.last_name, ua.birth_date, ua.email, ua.phone, ua.created
        from group_member gm
            join user_account ua on ua.id = gm.user_account
        where gm.user_group = $1
        order by ua.first_name, ua.last_name
        ")
        .bind(group_id)
        .map(|row: PgRow| GroupMember {
            id: row.get("member_id"),
            user_group: Some(UserGroup::new_user_group(row.get("user_group"))),
            user_account: Some(UserAccount {
                id: row.get("user_id"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                birth_date: row.get("birth_date"),
                email: row.get("email"),
                phone: row.get("phone"),
                password: None,
                created: row.get("created"),
                modified: None,
            }),
            created_date: row.get("created_date"),
            created_by: row.get::<Option<i32>, _>("created_by").map(UserAccount::new_user_account),
        })
        .fetch_all(conn)
        .await
}

pub async fn create_group(conn: &PgPool, name: &str, group_type: GroupType, creator_id: i32) -> Result<UserGroup, Error> {
    let mut tx = conn.begin().await?;

    let group = sqlx::query(r#"
        insert into user_group (name, type, created_by)
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
    sqlx::query("insert into group_member (user_group, user_account) values ($1, $2)")
        .bind(group_id)
        .bind(user_account_id)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn update_user_account(conn: &PgPool, user_account: &UserAccount) -> Result<(), Error> {
    sqlx::query("
        update user_account
        set first_name = $1, last_name = $2, birth_date = $3, email = $4, phone = $5, modified = now()
        where id = $6
    ")
        .bind(&user_account.first_name)
        .bind(&user_account.last_name)
        .bind(user_account.birth_date)
        .bind(&user_account.email)
        .bind(&user_account.phone)
        .bind(user_account.id)
        .execute(conn)
        .await?;

    Ok(())
}

fn row_to_good(row: &PgRow) -> Good {
    Good {
        id: row.get("id"),
        user_group: row.get::<Option<i32>, _>("user_group").map(UserGroup::new_user_group),
        name: row.get("name"),
        description: row.get("description"),
        created_by: row.get::<Option<i32>, _>("created_by").map(UserAccount::new_user_account),
        created_date: row.get("created_date"),
    }
}

// Goods are visible to the user who created them and to the members of the group they belong to.
const GOOD_ACCESS: &str = "
    (g.created_by = $1 or g.user_group in (select user_group from group_member where user_account = $1))
";

pub async fn get_goods(conn: &PgPool, user_account_id: i32) -> Result<Vec<Good>, Error> {
    sqlx::query(&format!("
        select g.*
        from good g
        where {}
        order by g.name
        ", GOOD_ACCESS))
        .bind(user_account_id)
        .map(|row: PgRow| row_to_good(&row))
        .fetch_all(conn)
        .await
}

pub async fn get_good(conn: &PgPool, id: i32, user_account_id: i32) -> Result<Good, Error> {
    sqlx::query(&format!("
        select g.*
        from good g
        where g.id = $2 and {}
        ", GOOD_ACCESS))
        .bind(user_account_id)
        .bind(id)
        .map(|row: PgRow| row_to_good(&row))
        .fetch_one(conn)
        .await
}

pub async fn create_good(conn: &PgPool, good: &Good) -> Result<Good, Error> {
    sqlx::query("
        insert into good (user_group, name, description, created_by)
        values ($1, $2, $3, $4)
        returning *
        ")
        .bind(good.user_group.as_ref().map(|g| g.id))
        .bind(&good.name)
        .bind(&good.description)
        .bind(good.created_by.as_ref().map(|ua| ua.id))
        .map(|row: PgRow| row_to_good(&row))
        .fetch_one(conn)
        .await
}

pub async fn update_good(conn: &PgPool, good: &Good, user_account_id: i32) -> Result<u64, Error> {
    let result = sqlx::query(&format!("
        update good g
        set name = $3, description = $4
        where g.id = $2 and {}
        ", GOOD_ACCESS))
        .bind(user_account_id)
        .bind(good.id)
        .bind(&good.name)
        .bind(&good.description)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

pub async fn delete_good(conn: &PgPool, id: i32, user_account_id: i32) -> Result<u64, Error> {
    let result = sqlx::query(&format!("
        delete from good g
        where g.id = $2 and {}
        ", GOOD_ACCESS))
        .bind(user_account_id)
        .bind(id)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

use sqlx::{Error, Row};
use sqlx::postgres::{PgRow, PgPool};
use crate::account::model::{UserGroup, UserAccount};
use crate::shopping::model::{Good, GoodLocation};

fn row_to_good(row: &PgRow) -> Good {
    Good {
        id: row.get("id"),
        user_group: row.get::<Option<i32>, _>("user_group").map(UserGroup::new_user_group),
        name: row.get("name"),
        description: row.get("description"),
        location: row.get::<Option<i32>, _>("location").map(GoodLocation::new_good_location),
        created_by: row.get::<Option<i32>, _>("created_by").map(UserAccount::new_user_account),
        created_date: row.get("created_date"),
    }
}

fn row_to_good_location(row: &PgRow) -> GoodLocation {
    GoodLocation {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        created_by: row.get::<Option<i32>, _>("created_by").map(UserAccount::new_user_account),
        created_date: row.get("created_date"),
    }
}

// Locations are visible to the user who created them and to the members of the groups that user belongs to.
const GOOD_LOCATION_ACCESS: &str = "
    (gl.created_by = $1 or gl.created_by in (
        select gm.user_account
        from group_member gm
        where gm.user_group in (select user_group from group_member where user_account = $1)))
";

pub async fn get_good_locations(conn: &PgPool, user_account_id: i32) -> Result<Vec<GoodLocation>, Error> {
    sqlx::query(&format!("
        select gl.*
        from good_location gl
        where {}
        order by gl.name
        ", GOOD_LOCATION_ACCESS))
        .bind(user_account_id)
        .map(|row: PgRow| row_to_good_location(&row))
        .fetch_all(conn)
        .await
}

pub async fn create_good_location(conn: &PgPool, location: &GoodLocation) -> Result<GoodLocation, Error> {
    sqlx::query("
        insert into good_location (name, description, created_by)
        values ($1, $2, $3)
        returning *
        ")
        .bind(&location.name)
        .bind(&location.description)
        .bind(location.created_by.as_ref().map(|ua| ua.id))
        .map(|row: PgRow| row_to_good_location(&row))
        .fetch_one(conn)
        .await
}

pub async fn update_good_location(conn: &PgPool, location: &GoodLocation, user_account_id: i32) -> Result<u64, Error> {
    let result = sqlx::query(&format!("
        update good_location gl
        set name = $3, description = $4
        where gl.id = $2 and {}
        ", GOOD_LOCATION_ACCESS))
        .bind(user_account_id)
        .bind(location.id)
        .bind(&location.name)
        .bind(&location.description)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

pub async fn delete_good_location(conn: &PgPool, id: i32, user_account_id: i32) -> Result<u64, Error> {
    let result = sqlx::query(&format!("
        delete from good_location gl
        where gl.id = $2 and {}
        ", GOOD_LOCATION_ACCESS))
        .bind(user_account_id)
        .bind(id)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

// Goods are visible to the user who created them and to the members of the group they belong to.
const GOOD_ACCESS: &str = "
    (g.created_by = $1 or g.user_group in (select user_group from group_member where user_account = $1))
";

// Lists the goods stored in the given location, or the goods without a location when location_id is None.
pub async fn get_goods(conn: &PgPool, user_account_id: i32, location_id: Option<i32>) -> Result<Vec<Good>, Error> {
    sqlx::query(&format!("
        select g.*, gl.name as location_name
        from good g
            left join good_location gl on gl.id = g.location
        where g.location is not distinct from $2 and {}
        order by g.name
        ", GOOD_ACCESS))
        .bind(user_account_id)
        .bind(location_id)
        .map(|row: PgRow| {
            let mut good = row_to_good(&row);
            if let Some(location) = good.location.as_mut() {
                location.name = row.get("location_name");
            }
            good
        })
        .fetch_all(conn)
        .await
}

pub async fn get_good(conn: &PgPool, id: i32, user_account_id: i32) -> Result<Good, Error> {
    sqlx::query(&format!("
        select g.*, ug.name as group_name, gl.name as location_name, ua.first_name, ua.last_name
        from good g
            left join user_group ug on ug.id = g.user_group
            left join good_location gl on gl.id = g.location
            left join user_account ua on ua.id = g.created_by
        where g.id = $2 and {}
        ", GOOD_ACCESS))
        .bind(user_account_id)
        .bind(id)
        .map(|row: PgRow| {
            let mut good = row_to_good(&row);
            if let Some(user_group) = good.user_group.as_mut() {
                user_group.name = row.get("group_name");
            }
            if let Some(location) = good.location.as_mut() {
                location.name = row.get("location_name");
            }
            if let Some(created_by) = good.created_by.as_mut() {
                created_by.first_name = row.get("first_name");
                created_by.last_name = row.get("last_name");
            }
            good
        })
        .fetch_one(conn)
        .await
}

pub async fn create_good(conn: &PgPool, good: &Good) -> Result<Good, Error> {
    sqlx::query("
        insert into good (user_group, name, description, location, created_by)
        values ($1, $2, $3, $4, $5)
        returning *
        ")
        .bind(good.user_group.as_ref().map(|g| g.id))
        .bind(&good.name)
        .bind(&good.description)
        .bind(good.location.as_ref().map(|l| l.id))
        .bind(good.created_by.as_ref().map(|ua| ua.id))
        .map(|row: PgRow| row_to_good(&row))
        .fetch_one(conn)
        .await
}

pub async fn update_good(conn: &PgPool, good: &Good, user_account_id: i32) -> Result<u64, Error> {
    let result = sqlx::query(&format!("
        update good g
        set name = $3, description = $4, location = $5
        where g.id = $2 and {}
        ", GOOD_ACCESS))
        .bind(user_account_id)
        .bind(good.id)
        .bind(&good.name)
        .bind(&good.description)
        .bind(good.location.as_ref().map(|l| l.id))
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

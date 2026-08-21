use std::str::FromStr;

use rocket::http::CookieJar;
use sea_orm::{DatabaseConnection, EntityTrait};
use uuid::Uuid;
use crate::entity::users;



//true -> is admin / false -> not admin
pub async fn check(cookies: &CookieJar<'_>, db: &DatabaseConnection) -> bool { 
    let val = match cookies.get(crate::auth::UUID_COOKIE_NAME).map(|c| c.value()) {
        Some(a) => a,
        None => {
            return false;
        },
    };

    let uuid = match Uuid::from_str(val) {
        Ok(a) => a,
        Err(_) => {
            return false;
        },
    };

    match users::Entity::find_by_id(uuid).one(db).await {
        Ok(Some(a)) => {
            a.is_admin
        },
        _ => {
            false
        }
    }
}

pub async fn check_pick(cookies: &CookieJar<'_>, db: &DatabaseConnection) -> bool { 
    let val = match cookies.get(crate::auth::UUID_COOKIE_NAME).map(|c| c.value()) {
        Some(a) => a,
        None => {
            return false;
        },
    };

    let uuid = match Uuid::from_str(val) {
        Ok(a) => a,
        Err(_) => {
            return false;
        },
    };

    match users::Entity::find_by_id(uuid).one(db).await {
        Ok(Some(a)) => {
            a.is_pick
        },
        _ => {
            false
        }
    }
}
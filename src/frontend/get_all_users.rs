use rocket::{State, serde::json::Json};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde_json::Value;

use crate::{entity::users, frontend::ApiResult};




#[rocket_okapi::openapi]
#[get("/api/misc/get_all_users")]
pub async fn get_all_users(db: &State<DatabaseConnection>) -> Json<ApiResult<Vec<String>>> {
    let users: Vec<String> = match users::Entity::find().all(db.inner()).await {
        Ok(a) => {
            a.into_iter().map(|x| x.name).collect()
        },
        Err(a) => {
            return Json(ApiResult::Error(format!("Database Error: {a}")));
        },
    };

    Json(ApiResult::Success(users))
}
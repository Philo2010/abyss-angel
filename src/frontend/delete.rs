use rocket::{State, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use serde::Serialize;
use serde_json::Value;
use rocket_okapi::JsonSchema;

use crate::{SETTINGS, auth, backenddb::game::delete_game, frontend::ApiResult};


#[rocket_okapi::openapi]
#[get("/api/delete/<id>")]
pub async fn delete_scout(id: i32, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    match delete_game(id, db).await {
        Ok(_) => {
            return Json(ApiResult::Success("OK".to_string()));
        },
        Err(a) => {
            return Json(ApiResult::Error(a.to_string()));
        },
    }
}
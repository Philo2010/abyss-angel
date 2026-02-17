use rocket::{State, http::CookieJar, serde::json::Json};
use sea_orm::DatabaseConnection;

use crate::{auth, backenddb::game::delete_game, frontend::ApiResult};


#[rocket_okapi::openapi]
#[get("/api/delete/<id>")]
pub async fn delete_scout(id: i32, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    match delete_game(id, db).await {
        Ok(_) => {
            Json(ApiResult::Success("OK".to_string()))
        },
        Err(a) => {
            Json(ApiResult::Error(a.to_string()))
        },
    }
}
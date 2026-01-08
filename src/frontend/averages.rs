
use rocket::{State, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::{SETTINGS, auth, backenddb::game::{GamesAvg, average_game}, frontend::ApiResult, sexymac};

#[rocket_okapi::openapi]
#[get("/api/averages/<event>")]
pub async fn averages(event: &str, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<Vec<GamesAvg>>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    let res = match average_game(&event.to_string(), db).await {
        Ok(a) => a,
        Err(a) => {
            return Json(ApiResult::Error(a.to_string()));
        }
    };


    Json(ApiResult::Success(res))
}
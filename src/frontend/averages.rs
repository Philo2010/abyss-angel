
use rocket::{State, http::CookieJar, serde::json::Json};
use sea_orm::DatabaseConnection;

use crate::{auth, backenddb::game::{TeamAvg, average_game}, frontend::ApiResult};

#[rocket_okapi::openapi]
#[get("/api/averages/<event>")]
pub async fn averages(event: &str, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<Vec<TeamAvg>>> {
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
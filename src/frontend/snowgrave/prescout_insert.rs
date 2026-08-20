use rocket::State;
use rocket::http::CookieJar;
use rocket::post;
use rocket::serde::json::Json;
use sea_orm::DatabaseConnection;

use crate::auth;
use crate::frontend::ApiResult;
use crate::snowgrave::prescout::PrescoutInsert;

/// Store admin-entered prescout data. Unlike the other snowgrave insert routes
/// this one is admin-gated — it writes straight to final storage with no check
/// phase in front of it.
#[rocket_okapi::openapi]
#[post("/api/prescout/insert", data = "<body>")]
pub async fn prescout_insert(
    body: Json<PrescoutInsert>,
    db: &State<DatabaseConnection>,
    cookies: &CookieJar<'_>,
) -> Json<ApiResult<String>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }

    let user = match auth::get_by_cookie::get(cookies) {
        Some(a) => a,
        None => {
            return Json(ApiResult::Error("Not logged in!".to_string()));
        }
    };

    match crate::snowgrave::prescout::insert_prescout(user, body.into_inner(), db).await {
        Ok(id) => Json(ApiResult::Success(format!("Prescout stored (id {id})"))),
        Err(a) => Json(ApiResult::Error(format!("Database Error while inserting: {a}"))),
    }
}

use rocket::State;
use rocket::http::CookieJar;
use rocket::post;
use rocket::serde::json::Json;
use sea_orm::DatabaseConnection;

use crate::auth;
use crate::frontend::ApiResult;
use crate::snowgrave::prescout::PrescoutInsert;

/// Store admin-entered prescout data. Unlike the other snowgrave insert routes
/// this one writes straight to final storage with no check phase in front of it.
/// It is open to anonymous guests: with no login cookie the stored header simply
/// has an empty user vector (shown as GUEST in the UI).
#[rocket_okapi::openapi]
#[post("/api/prescout/insert", data = "<body>")]
pub async fn prescout_insert(
    body: Json<PrescoutInsert>,
    db: &State<DatabaseConnection>,
    cookies: &CookieJar<'_>,
) -> Json<ApiResult<String>> {
    /* //No longer admin
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }*/

    // Prescout can be entered by an anonymous guest, so a missing cookie is not
    // an error — the header just ends up with an empty user vector.
    let user = auth::get_by_cookie::get(cookies);

    match crate::snowgrave::prescout::insert_prescout(user, body.into_inner(), db).await {
        Ok(id) => Json(ApiResult::Success(format!("Prescout stored (id {id})"))),
        Err(a) => Json(ApiResult::Error(format!("Database Error while inserting: {a}"))),
    }
}

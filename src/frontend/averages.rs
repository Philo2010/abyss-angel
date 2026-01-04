
use rocket::{State, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::{SETTINGS, auth, backenddb::game::{GamesAvg, average_game}, sexymac};

#[derive(Responder)]
pub enum AverageResponse {
    #[response(status = 200)]
    Success(Json<Vec<GamesAvg>>),
    #[response(status = 400)]
    Error(Json<String>),
}

#[get("/averages/<event>")]
pub async fn averages(event: &str, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> AverageResponse {
    if !auth::check::check(cookies, db).await {
        return AverageResponse::Error(Json("Need to be admin!".to_string()));
    }
    let res = match average_game(&event.to_string(), db).await {
        Ok(a) => a,
        Err(a) => {
            return AverageResponse::Error(Json(a.to_string()));
        }
    };


    AverageResponse::Success(Json(res))
}
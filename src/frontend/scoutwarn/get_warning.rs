use rocket::{State, http::CookieJar, serde::json::Json};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::{auth::get_by_cookie, frontend::scoutwarn, scoutwarn::get_warning::ReturnWarning};


#[derive(Responder)]
pub enum ScoutWarnResult {
    #[response(status = 200)]
    Success(Json<Vec<ReturnWarning>>),
    #[response(status = 400)]
    Error(Json<String>),
}


#[get("/api/scoutwarn/getall")]
pub async fn get_scoutwarn(db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> ScoutWarnResult {
    let uuid = match get_by_cookie::get(cookies) {
        Some(a) => a,
        None => {
            return ScoutWarnResult::Error(Json("Not logined in!".to_string()));
        },
    };

    match crate::scoutwarn::get_warning::get_warning(uuid, db).await {
        Ok(a) => {
            return ScoutWarnResult::Success(Json(a));
        },
        Err(a) => {
            return ScoutWarnResult::Error(Json(format!("Database error while trying to get games: {a}")));
        },
    }
}
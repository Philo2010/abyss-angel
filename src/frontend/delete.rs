use rocket::{State, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::{SETTINGS, auth, backenddb::game::delete_game, frontend::snowgrave::queue::QueueResponder};

#[derive(Responder)]
pub enum DeleteResponse {
    #[response(status = 200)]
    Success(Json<String>),
    #[response(status = 400)]
    Error(Json<String>),
}


#[get("/delete/<id>")]
pub async fn delete_scout(id: i32, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> DeleteResponse {
    if !auth::check::check(cookies, db).await {
        return DeleteResponse::Error(Json("Need to be admin!".to_string()));
    }
    match delete_game(id, db).await {
        Ok(_) => {
            return DeleteResponse::Success(Json("OK".to_string()));
        },
        Err(a) => {
            return DeleteResponse::Error(Json(a.to_string()));
        },
    }
}
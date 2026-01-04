use rocket::State;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_json::Value;
use crate::auth;

use crate::pit::pit::{PitEditSpecific, PitGet, PitHeaderInsert, PitInsert, PitInsertsSpecific, pit_edit, pit_get};
use crate::{SETTINGS, backenddb::game::{GamesAvg, average_game}, sexymac};

#[derive(Responder)]
pub enum PitEditResponse {
    #[response(status = 200)]
    Success(Json<String>),
    #[response(status = 400)]
    Error(Json<String>),
}

#[derive(Deserialize)]
pub struct PitHeaderGetFront { 
    pub team: i32,
    pub is_ab_team: bool,
    pub event_code: String,
}



#[post("/apt/pit/edit/<id>", data ="<data>")]
pub async fn edit(id: i32, data: Json<PitEditSpecific>,  db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> PitEditResponse {
    if !auth::check::check(cookies, db).await {
        return PitEditResponse::Error(rocket::serde::json::Json("Need to be admin!".to_string()));
    }

    match pit_edit(data.into_inner(), db, id).await {
        Ok(a) => {
            return PitEditResponse::Success(Json("Edit is done!".to_string()));
        },
        Err(a) => {
            return PitEditResponse::Error(Json(format!("DB Error trying to insert data: {a}")));
        },
    }
}
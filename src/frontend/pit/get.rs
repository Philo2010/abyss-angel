use rocket::form::Form;
use rocket::{State, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use crate::auth;

use crate::pit::pit::{PitGet, PitHeaderInsert, PitInsert, PitInsertsSpecific, pit_get};
use crate::{SETTINGS, backenddb::game::{GamesAvg, average_game}, sexymac};

#[derive(Responder)]
pub enum PitGetResponse {
    #[response(status = 200)]
    Success(Json<PitGet>),
    #[response(status = 400)]
    Error(Json<String>),
}

#[derive(Deserialize)]
pub struct PitHeaderGetFront { 
    pub team: i32,
    pub is_ab_team: bool,
    pub event_code: String,
}



#[post("/apt/pit/get", data ="<data>")]
pub async fn get(data: Json<PitHeaderGetFront>,  db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> PitGetResponse {
    if !auth::check::check(cookies, db).await {
        return PitGetResponse::Error(Json("Need to be admin!".to_string()));
    }

    match pit_get(data.team, data.is_ab_team, &data.event_code, db).await {
        Ok(a) => {
            return PitGetResponse::Success(Json(a));
        },
        Err(a) => {
            return PitGetResponse::Error(Json(format!("Database error while trying to fetch game data: {a}")));
        }
    };
}
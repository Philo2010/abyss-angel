use rocket::form::Form;
use rocket::{State, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use crate::auth;

use crate::pit::pit::{PitHeaderInsert, PitInsert, PitInsertsSpecific};
use crate::{SETTINGS, backenddb::game::{GamesAvg, average_game}, sexymac};

#[derive(Responder)]
pub enum PitInsertResponse {
    #[response(status = 200)]
    Success(Json<i32>),
    #[response(status = 400)]
    Error(Json<String>),
}

#[derive(Deserialize)]
pub struct PitHeaderInsertFront { 
    pub team: i32,
    pub is_am_team: bool,
    pub event_code: String,
    pub pit: PitInsertsSpecific
}



#[post("/apt/pit/insert", data ="<data>")]
pub async fn insert(data: Json<PitHeaderInsertFront>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> PitInsertResponse {
    let user = match auth::get_by_cookie::get(cookies) {
        Some(a) => {a},
        None => {
            return PitInsertResponse::Error(Json("Not logined in!".to_string()));
        }
    };
    let insert_header_data = PitHeaderInsert {
        user,
        team: data.team,
        is_am_team: data.is_am_team,
        event_code: data.event_code.clone()
    };
    let insert_data = PitInsert { 
        header: insert_header_data, 
        pit: data.pit.clone(),
    };
    match crate::pit::pit::pit_insert(insert_data, db).await {
        Ok(a) => {
            return PitInsertResponse::Success(Json(a));
        }, 
        Err(a) => {
            return PitInsertResponse::Error(Json(format!("Failed to insert into db: {a}")));
        }
    };
}
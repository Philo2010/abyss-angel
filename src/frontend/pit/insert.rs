use rocket::form::Form;
use rocket::{State, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use rocket_okapi::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use crate::auth;

use crate::frontend::ApiResult;
use crate::pit::pit::{PitHeaderInsert, PitInsert, PitInsertsSpecific};
use crate::{SETTINGS, backenddb::game::{GamesAvg, average_game}, sexymac};


#[derive(Deserialize, JsonSchema)]
pub struct PitHeaderInsertFront { 
    pub team: i32,
    pub is_ab_team: bool,
    pub event_code: String,
    pub pit: PitInsertsSpecific
}


#[rocket_okapi::openapi]
#[post("/api/pit/insert", data ="<data>")]
pub async fn insert(data: Json<PitHeaderInsertFront>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<i32>> {
    let user = match auth::get_by_cookie::get(cookies) {
        Some(a) => {a},
        None => {
            return Json(ApiResult::Error("Not logined in!".to_string()));
        }
    };
    let insert_header_data = PitHeaderInsert {
        user,
        team: data.team,
        is_ab_team: data.is_ab_team,
        event_code: data.event_code.clone()
    };
    let insert_data = PitInsert { 
        header: insert_header_data, 
        pit: data.pit.clone(),
    };
    match crate::pit::pit::pit_insert(insert_data, db).await {
        Ok(a) => {
            return Json(ApiResult::Success(a));
        }, 
        Err(a) => {
            return Json(ApiResult::Error(format!("Failed to insert into db: {a}")));
        }
    };
}
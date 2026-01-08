use rocket::State;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use rocket_okapi::JsonSchema;
use serde_json::Value;
use crate::auth;

use crate::frontend::ApiResult;
use crate::pit::pit::{PitEditSpecific, PitGet, PitHeaderInsert, PitInsert, PitInsertsSpecific, pit_edit, pit_get};
use crate::{SETTINGS, backenddb::game::{GamesAvg, average_game}, sexymac};

#[derive(Deserialize, JsonSchema)]
pub struct PitHeaderGetFront { 
    pub team: i32,
    pub is_ab_team: bool,
    pub event_code: String,
}


#[rocket_okapi::openapi]
#[post("/api/pit/edit/<id>", data ="<data>")]
pub async fn edit_pit(id: i32, data: Json<PitEditSpecific>,  db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }

    match pit_edit(data.into_inner(), db, id).await {
        Ok(_a) => {
            return Json(ApiResult::Success("Edit is done!".to_string()));
        },
        Err(a) => {
            return Json(ApiResult::Error(format!("DB Error trying to insert data: {a}")));
        },
    }
}
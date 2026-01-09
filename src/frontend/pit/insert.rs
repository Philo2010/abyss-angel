use rocket::form::Form;
use rocket::{State, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use rocket_okapi::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use crate::{auth, pit};

use crate::frontend::ApiResult;
use crate::pit::pit::{PitHeaderInsert, PitInsert, PitInsertsSpecific, pit_insert};
use crate::{SETTINGS, backenddb::game::{GamesAvg, average_game}, sexymac};
use crate::pit::pit_insert::PitInsertForm;





#[rocket_okapi::openapi]
#[post("/api/pit/insert", data ="<data>")]
pub async fn insert(data: Json<PitInsertForm>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    match pit::pit_insert::pit_insert(db, data.into_inner()).await {
        Ok(a) => {
            return Json(ApiResult::Success("Done!".to_string()));
        },
        Err(a) => {
            return Json(ApiResult::Error(format!("Database Error: {a}")))
        },
    }
}
use chrono::{DateTime, Local};
use rocket::form::Form;
use rocket::{State, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use schemars::JsonSchema;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use crate::auth::{self, get_by_cookie};

use crate::auth::get_by_user::{AuthGetUuidError, get_by_uuid};
use crate::auth::get_by_cookie::get;
use crate::frontend::{ApiResult, pit};
use crate::pit::get_scouters_pit::PitScouterInstance;
use crate::pit::pit::{PitGet, PitHeaderInsert, PitInsert, PitInsertsSpecific, PitSpecific, pit_get};
use crate::setting::setevent::get_event_inner;
use crate::{SETTINGS, backenddb::game::{GamesAvg, average_game}, sexymac};



#[rocket_okapi::openapi]
#[post("/api/pit/get_scouter")]
pub async fn get_for_scout(db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<Vec<PitScouterInstance>>> {
    let uuid = match get_by_cookie::get(cookies) {
        Some(a) => a,
        None => {
            return Json(ApiResult::Error("Please login".to_string()));
        },
    };

    let event = match get_event_inner(db).await {
        Ok(a) => {
            a
        },
        Err(a) => {
            return Json(ApiResult::Error(format!("Database Error while trying to get event: {a}")));
        },
    };

    let games = match crate::pit::get_scouters_pit::get_scouters_pit(uuid, &event, db).await {
        Ok(a) => {
            a 
        },
        Err(a) => {
            return Json(ApiResult::Error(format!("Database Error while trying to get games: {a}")));
        },
    };

    Json(ApiResult::Success(games))
}
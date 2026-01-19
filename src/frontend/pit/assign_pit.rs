use rocket::State;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use rocket_okapi::JsonSchema;
use serde_json::Value;
use uuid::Uuid;
use crate::auth;

use crate::auth::get_by_user::get_by_username;
use crate::frontend::ApiResult;
use crate::pit::assign_pit_scouts::{self, AssignScoutForm};
use crate::pit::pit::{PitEditSpecific, PitGet, PitHeaderInsert, PitInsert, PitInsertsSpecific, pit_edit, pit_get};
use crate::{SETTINGS, backenddb::game::{GamesAvg, average_game}, sexymac};

#[derive(Deserialize, JsonSchema)]
pub struct AssignScoutFormButCool {
    scouts: Vec<String>,
    //first you is the index, next is the id to pit_upcoming
    asignment: Vec<PitAssigment>
}

#[derive(Deserialize, JsonSchema)]
pub struct PitAssigment {
    index: usize,
    upcomingid: i32
}

#[rocket_okapi::openapi]
#[post("/api/pit/assign", data ="<data>")]
pub async fn assign_pit(data: Json<AssignScoutFormButCool>,  db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }

    //convert String to Uuid
    let mut scouts_uuid: Vec<Uuid> = Vec::new();
    for scout in &data.scouts {
        let uuid = match get_by_username(scout, db).await {
            Ok(a) => a,
            Err(a) => {
                match a {
                    auth::get_by_user::AuthGetUuidError::UserIsNotHere => {
                        return Json(ApiResult::Error("Could not find user!".to_string()));
                    },
                    auth::get_by_user::AuthGetUuidError::DatabaseError(db_err) => {
                        return Json(ApiResult::Success(format!("Database Error: {db_err}")));
                    },
                }
            },
        };
        scouts_uuid.push(uuid);
    }

    let form = AssignScoutForm { 
        scouts: scouts_uuid,
        asignment: data.asignment.iter().map(|x| {
            (x.index, x.upcomingid)
        }).collect()
    };
    let res = match assign_pit_scouts::assign_pit_scouts(db, form).await {
        Ok(a) => a,
        Err(a) => {
            return Json(ApiResult::Error(format!("Database Error: {a}")));
        },
    };

    Json(ApiResult::Success("Done!".to_string()))
}
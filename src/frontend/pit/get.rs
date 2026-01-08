use chrono::{DateTime, Local};
use rocket::form::Form;
use rocket::{State, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use schemars::JsonSchema;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use crate::auth;

use crate::auth::get_by_user::{AuthGetUuidError, get_by_uuid};
use crate::frontend::ApiResult;
use crate::pit::pit::{PitGet, PitHeaderInsert, PitInsert, PitInsertsSpecific, PitSpecific, pit_get};
use crate::{SETTINGS, backenddb::game::{GamesAvg, average_game}, sexymac};

#[derive(Deserialize, JsonSchema)]
pub struct PitHeaderGetFront { 
    pub team: i32,
    pub is_ab_team: bool,
    pub event_code: String,
}


#[derive(Serialize, rocket_okapi::JsonSchema)]
struct FullHeaderSend {
    pub id: i32,
    pub user: String,
    pub team: i32,
    pub is_ab_team: bool,
    pub event_code: String,
    pub created_at: DateTime<Local>,
}

#[derive(Serialize, JsonSchema)]
pub struct PitGetSend {
    pub header: FullHeaderSend,
    pub pit: PitSpecific
}
async fn to_pit_get_send(value: PitGet, db: &DatabaseConnection) -> Result<PitGetSend, AuthGetUuidError> {
    let user_string = get_by_uuid(&value.header.user, db).await?;

    Ok(PitGetSend {
        header: FullHeaderSend {
            id: value.header.id,
            user: user_string,
            team: value.header.team,
            is_ab_team: value.header.is_ab_team,
            event_code: value.header.event_code,
            created_at: value.header.created_at
        },
        pit: value.pit,
    })
}

#[rocket_okapi::openapi]
#[post("/api/pit/get", data ="<data>")]
pub async fn get(data: Json<PitHeaderGetFront>,  db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<PitGetSend>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }

    match pit_get(data.team, data.is_ab_team, &data.event_code, db).await {
        Ok(a) => {
            let send = match to_pit_get_send(a, db).await {
                Ok(a) => a,
                Err(e) => {
                    match e {
                        AuthGetUuidError::UserIsNotHere => {
                            return Json(ApiResult::Error("Could not find user!".to_string()));
                        },
                        AuthGetUuidError::DatabaseError(db_err) => {
                            return Json(ApiResult::Error(format!("Failed to get username: {db_err}")));
                        },
                    }
                },
            };
            return Json(ApiResult::Success(send));
        },
        Err(a) => {
            return Json(ApiResult::Error(format!("Database error while trying to fetch game data: {a}")));
        }
    };
}
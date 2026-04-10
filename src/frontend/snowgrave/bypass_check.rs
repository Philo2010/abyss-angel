use rocket::State;
use rocket::serde::json::Json;
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::frontend::ApiResult;
use crate::snowgrave::check_system::nobonoko::{self, Alliance};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct BypassCheckRequest {
    pub game_id: i32,
    pub alliance: AllianceFront,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AllianceFront {
    Red,
    Blue,
}

#[rocket_okapi::openapi]
#[post("/api/snowgrave/bypass_check", data = "<data>")]
pub async fn bypass_check(data: Json<BypassCheckRequest>, db: &State<DatabaseConnection>) -> Json<ApiResult<String>> {
    let alliance = match data.alliance {
        AllianceFront::Red => Alliance::Red,
        AllianceFront::Blue => Alliance::Blue,
    };

    match nobonoko::bypass_check(data.game_id, alliance, db.inner()).await {
        Ok(_) => Json(ApiResult::Success("Override submitted successfully".to_string())),
        Err(e) => Json(ApiResult::Error(format!("Override failed: {e}"))),
    }
}

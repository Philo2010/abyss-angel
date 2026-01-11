use rocket::State;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::post;
use rocket::serde::json::Json;
use rocket_dyn_templates::{Template, context};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::backenddb::game::{GamesGraph, graph_game};
use crate::frontend::ApiResult;
use crate::snowgrave::insert_scout_data::InsertSnow;
use crate::{SETTINGS, auth, sexymac};





#[rocket_okapi::openapi]
#[post("/api/scout/insert", data = "<body>")]
pub async fn scout_insert(body: Json<InsertSnow>, db: &State<DatabaseConnection>) -> Json<ApiResult<String>> {
    match crate::snowgrave::insert_scout_data::insert_scout(db, body.into_inner()).await {
        Ok(_) => {
            Json(ApiResult::Success("Done!".to_string()))
        },
        Err(a) => {
            Json(ApiResult::Error(format!("Database Error while inserting: {a}")))
        },
    }
}
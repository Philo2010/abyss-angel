use rocket::{State, serde::json::Json};
use sea_orm::DatabaseConnection;

use crate::{frontend::ApiResult, pit::{self, pit_insert::PitInsertForm}};







#[rocket_okapi::openapi]
#[post("/api/pit/insert", data ="<data>")]
pub async fn insert(data: Json<PitInsertForm>, db: &State<DatabaseConnection>) -> Json<ApiResult<String>> {
    match pit::pit_insert::pit_insert(db, data.into_inner()).await {
        Ok(_) => {
            Json(ApiResult::Success("Done!".to_string()))
        },
        Err(a) => {
            Json(ApiResult::Error(format!("Database Error: {a}")))
        },
    }
}
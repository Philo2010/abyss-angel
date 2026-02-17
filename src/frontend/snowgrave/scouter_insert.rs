use rocket::State;
use rocket::post;
use rocket::serde::json::Json;
use sea_orm::DatabaseConnection;

use crate::frontend::ApiResult;
use crate::snowgrave::insert_scout_data::InsertSnow;





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
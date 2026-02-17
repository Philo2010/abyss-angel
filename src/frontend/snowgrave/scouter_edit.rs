use rocket::State;
use rocket::post;
use rocket::serde::json::Json;
use sea_orm::DatabaseConnection;

use crate::frontend::ApiResult;
use crate::snowgrave::snowgrave_edit_scouter::EditSnow;





#[rocket_okapi::openapi]
#[post("/api/scout/edit", data = "<body>")]
pub async fn scout_edit(body: Json<EditSnow>, db: &State<DatabaseConnection>) -> Json<ApiResult<String>> {
    match crate::snowgrave::snowgrave_edit_scouter::edit_scouter(body.into_inner(), db).await {
        Ok(_) => {
            Json(ApiResult::Success("Done!".to_string()))
        },
        Err(a) => {
            Json(ApiResult::Error(format!("Database Error while inserting: {a}")))
        },
    }
}
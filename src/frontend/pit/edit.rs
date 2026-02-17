use rocket::State;
use rocket::serde::json::Json;
use sea_orm::DatabaseConnection;
use crate::pit::pit::PitEditSpecific;
use crate::pit::pit_edit::PitEditForm;
use crate::pit;

use crate::frontend::ApiResult;


#[rocket_okapi::openapi]
#[post("/api/pit/edit/<id>", data ="<data>")]
pub async fn edit_pit(id: i32, data: Json<PitEditSpecific>,  db: &State<DatabaseConnection>) -> Json<ApiResult<String>> {

    match pit::pit_edit::pit_edit(db, PitEditForm { pit_insert_id: id, pit: data.into_inner() }).await {
        Ok(_a) => {
            Json(ApiResult::Success("Edit is done!".to_string()))
        },
        Err(a) => {
            Json(ApiResult::Error(format!("DB Error trying to insert data: {a}")))
        },
    }
}
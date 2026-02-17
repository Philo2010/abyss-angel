use rocket::{State, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::frontend::ApiResult;



#[derive(Deserialize, JsonSchema)]
pub struct Forgive {
    id: i32
}

#[rocket_okapi::openapi]
#[post("/api/scoutwarn/forgive", data="<data>")]
pub async fn forgive_scoutwarn(db: &State<DatabaseConnection>, data: Json<Forgive>) -> Json<ApiResult<String>> {
    match crate::scoutwarn::forgive_warning::forgive_warning(data.id, db).await {
        Ok(_) => {
            return Json(ApiResult::Success("Forgiven".to_string()));
        },
        Err(a) => {
            Json(ApiResult::Error(format!("Database Error: {a}")))
        },
    }
}
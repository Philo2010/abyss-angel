use rocket::{State, serde::json::Json};
use sea_orm::DatabaseConnection;
use crate::{frontend::ApiResult, snowgrave::{self, insert_mvp_data::MvpInsert}};


#[rocket_okapi::openapi]
#[post("/api/mvp/insert", data = "<body>")]
pub async fn mvp_insert(body: Json<MvpInsert>, db: &State<DatabaseConnection>) -> Json<ApiResult<String>> {

    match snowgrave::insert_mvp_data::insert_mvp_data(body.into_inner(), db).await {
        Ok(a) => {
            Json(ApiResult::Success("Done!".to_string()))
        },
        Err(a) => {
            Json(ApiResult::Error(format!("Database error: {a}")))
        },
    }
}
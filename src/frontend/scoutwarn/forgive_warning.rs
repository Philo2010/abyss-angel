use rocket::{State, http::CookieJar, serde::json::Json};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_json::Value;


#[derive(Responder)]
pub enum ForgiveResponder {
    #[response(status = 200)]
    Success(Json<String>),
    #[response(status = 400)]
    Error(Json<String>),
}

#[derive(Deserialize)]
pub struct Forgive {
    id: i32
}

#[post("/api/scoutwarn/forgive", data="<data>")]
pub async fn get_scoutwarn(db: &State<DatabaseConnection>, data: Json<Forgive>) -> ForgiveResponder {
    match crate::scoutwarn::forgive_warning::forgive_warning(data.id, db).await {
        Ok(a) => {
            return ForgiveResponder::Success(Json("Forgiven".to_string()));
        },
        Err(a) => {
            return ForgiveResponder::Error(Json(format!("Database Error: {a}")));
        },
    };
}
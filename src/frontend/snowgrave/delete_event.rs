use rocket::{State, http::CookieJar, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::{auth, frontend::ApiResult, snowgrave::delete_event::delete_event};

#[derive(Deserialize, JsonSchema)]
pub struct DeleteEventInput {
    pub event: String,
}

#[rocket_okapi::openapi]
#[delete("/api/snowgrave/delete_event", data = "<data>")]
pub async fn delete_event_route(
    data: Json<DeleteEventInput>,
    db: &State<DatabaseConnection>,
    cookies: &CookieJar<'_>,
) -> Json<ApiResult<String>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }

    match delete_event(&data.event, db.inner()).await {
        Ok(_) => Json(ApiResult::Success(format!(
            "Deleted all data for event '{}'",
            data.event
        ))),
        Err(e) => Json(ApiResult::Error(format!("Database error: {e}"))),
    }
}

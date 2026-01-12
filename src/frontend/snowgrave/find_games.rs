use rocket::{State, http::{CookieJar, private::cookie}, serde::json::Json};
use rocket_okapi::JsonSchema;
use sea_orm::{DatabaseConnection};
use serde_json::Value;
use crate::{auth, frontend::{ApiResult, snowgrave}, snowgrave::datatypes::{GamePartial, GamePartialWithoutId}};



#[rocket_okapi::openapi]
#[get("/api/snowgrave/get_years")]
pub async fn get_years(cookies: &CookieJar<'_>, db: &State<DatabaseConnection>) -> Json<ApiResult<Vec<GamePartialWithoutId>>>  {

    let uuid = match auth::get_by_cookie::get(cookies) {
        Some(a) => a,
        None => {
            return Json(ApiResult::Error("Not login in".to_string()));
        },
    };
    let games: Vec<GamePartialWithoutId> = match crate::snowgrave::get_games_from_scouter::get_games_for_scouter(uuid, db.inner()).await {
        Ok(a) => a,
        Err(a) => {
            return Json(ApiResult::Error(format!("Could not find games: {a}")));
        },
    };

    Json(ApiResult::Success(games))
}
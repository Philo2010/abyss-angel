use rocket::{State, http::CookieJar, serde::json::Json};
use sea_orm::DatabaseConnection;

use crate::{auth, frontend::ApiResult, snowgrave::get_snowgrave_leader_board::{SnowScouterDataLeaderBoard, get_snowgrave_leader_board}};


#[rocket_okapi::openapi]
#[get("/api/snowgrave/leaderboard")]
pub async fn get_leaderboard(db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<Vec<SnowScouterDataLeaderBoard>>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    let res = match get_snowgrave_leader_board(db).await {
        Ok(a) => {
            Json(ApiResult::Success(a))
        },
        Err(a) => {
            Json(ApiResult::Error(format!("Database Error: {a}")))
        },
    };

    res
}
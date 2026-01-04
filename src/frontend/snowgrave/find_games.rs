use rocket::{State, http::{CookieJar, private::cookie}, serde::json::Json};
use sea_orm::{DatabaseConnection};
use serde_json::Value;
use crate::{auth, frontend::snowgrave, snowgrave::datatypes::GamePartial};

#[derive(Responder)]
pub enum GetGamesResponse {
    #[response(status = 200)]
    Success(Json<Vec<GamePartial>>),
    #[response(status = 400)]
    Error(Json<String>),
}


#[get("/api/snowgrave/get_years")]
pub async fn get_years(cookies: &CookieJar<'_>, db: &State<&DatabaseConnection>) -> GetGamesResponse  {

    let uuid = match auth::get_by_cookie::get(cookies) {
        Some(a) => a,
        None => {
            return GetGamesResponse::Error(Json("Not login in".to_string()))
        },
    };
    let games: Vec<GamePartial> = match crate::snowgrave::get_games_from_scouter::get_games_for_scouter(uuid, db.inner()).await {
        Ok(a) => a,
        Err(a) => {
            return GetGamesResponse::Error(Json(format!("Could not find games: {a}")));
        },
    };

    GetGamesResponse::Success(Json(games))
}
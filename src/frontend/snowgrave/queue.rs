//both normal and playoffs

use reqwest::Client;
use rocket::{State, data::FromData, http::CookieJar, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_json::Value;

use crate::{auth, frontend::ApiResult, snowgrave::{self, snowgrave_que::Blue2DBErr}};

#[derive(Deserialize, JsonSchema)]
pub struct QueueInput {
    pub event: String,
}

fn handle_blue_err(a: Blue2DBErr) -> Json<ApiResult<String>> {
    match a {
                snowgrave::snowgrave_que::Blue2DBErr::FailedToFindRightTourLevel(a) => {
                    return Json(ApiResult::Error(format!("Failed to find right tournament level: {a}")));
                },
                snowgrave::snowgrave_que::Blue2DBErr::FailedToParseTeam(parse_int_error) => {
                    return Json(ApiResult::Error(format!("Failed to parse a teams value: {parse_int_error}")));
                },
                snowgrave::snowgrave_que::Blue2DBErr::InvaildStation(a) => {
                    return Json(ApiResult::Error(format!("Invaild station id: {a}")));
                },
                snowgrave::snowgrave_que::Blue2DBErr::DbErr(db_err) => {
                    return Json(ApiResult::Error(format!("Database Error: {db_err}")));
                },
            }
}


#[rocket_okapi::openapi]
#[post("/api/snowgrave/queue_match", data="<data>")]
pub async fn queue(data: Json<QueueInput>, client: &State<Client>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }

    let tba_games = match snowgrave::blue::pull_from_blue(client, &data.event).await {
        Ok(a) => a,
        Err(a) => {
            return {
                Json(ApiResult::Error(format!("Failed to get matches from blue: {a}")))
            };
        },
    };

    let res = match snowgrave::snowgrave_que::queue_snow(tba_games, &data.event, client, db).await {
        Ok(a) => a,
        Err(a) => {
            return handle_blue_err(a);
        },
    };

    Json(ApiResult::Success("Able to add games!".to_string()))
}


#[rocket_okapi::openapi]
#[post("/api/snowgrave/queue_match_playoff", data="<data>")]
pub async fn queue_playoff(data: Json<QueueInput>, client: &State<Client>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    let tba_games = match snowgrave::blue::pull_from_blue(client, &data.event).await {
        Ok(a) => a,
        Err(a) => {
            return {
                Json(ApiResult::Error(format!("Failed to get matches from blue: {a}")))
            };
        },
    };

    let res = match snowgrave::snowgrave_que_only_playoff::queue_snow(tba_games, &data.event, client, db).await {
        Ok(a) => a,
        Err(a) => {
            return handle_blue_err(a);
        },
    };

    Json(ApiResult::Success("Able to add games!".to_string()))
}
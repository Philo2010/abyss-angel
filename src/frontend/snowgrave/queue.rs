//both normal and playoffs

use reqwest::Client;
use rocket::{State, data::FromData, http::CookieJar, serde::json::Json};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_json::Value;

use crate::{auth, snowgrave::{self, snowgrave_que::Blue2DBErr}};

#[derive(Responder)]
pub enum QueueResponder {
    #[response(status = 200)]
    Success(Json<String>),
    #[response(status = 400)]
    Error(Json<String>),
}

#[derive(Deserialize)]
pub struct QueueInput {
    pub event: String,
}

fn handle_blue_err(a: Blue2DBErr) -> QueueResponder {
    match a {
                snowgrave::snowgrave_que::Blue2DBErr::FailedToFindRightTourLevel(a) => {
                    return QueueResponder::Error(Json(format!("Failed to find right tournament level: {a}")));
                },
                snowgrave::snowgrave_que::Blue2DBErr::FailedToParseTeam(parse_int_error) => {
                    return QueueResponder::Error(Json(format!("Failed to parse a teams value: {parse_int_error}")));
                },
                snowgrave::snowgrave_que::Blue2DBErr::InvaildStation(a) => {
                    return QueueResponder::Error(Json(format!("Invaild station id: {a}")));
                },
                snowgrave::snowgrave_que::Blue2DBErr::DbErr(db_err) => {
                    return QueueResponder::Error(Json(format!("Database Error: {db_err}")));
                },
            }
}

#[post("/api/snowgrave/queue_match", data="<data>")]
pub async fn queue(data: Json<QueueInput>, client: &State<Client>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> QueueResponder {
    if !auth::check::check(cookies, db).await {
        return QueueResponder::Error(Json("Need to be admin!".to_string()));
    }

    let tba_games = match snowgrave::blue::pull_from_blue(client, &data.event).await {
        Ok(a) => a,
        Err(a) => {
            return {
                QueueResponder::Error(Json(format!("Failed to get matches from blue: {a}")))
            };
        },
    };

    let res = match snowgrave::snowgrave_que::queue_snow(tba_games, &data.event, client, db).await {
        Ok(a) => a,
        Err(a) => {
            return handle_blue_err(a);
        },
    };

    QueueResponder::Success(Json("Able to add games!".to_string()))
}

#[post("/api/snowgrave/queue_match_playoff", data="<data>")]
pub async fn queue_playoff(data: Json<QueueInput>, client: &State<Client>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> QueueResponder {
    if !auth::check::check(cookies, db).await {
        return QueueResponder::Error(Json("Need to be admin!".to_string()));
    }
    let tba_games = match snowgrave::blue::pull_from_blue(client, &data.event).await {
        Ok(a) => a,
        Err(a) => {
            return {
                QueueResponder::Error(Json(format!("Failed to get matches from blue: {a}")))
            };
        },
    };

    let res = match snowgrave::snowgrave_que_only_playoff::queue_snow(tba_games, &data.event, client, db).await {
        Ok(a) => a,
        Err(a) => {
            return handle_blue_err(a);
        },
    };

    QueueResponder::Success(Json("Able to add games!".to_string()))
}
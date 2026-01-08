
use rocket::State;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::post;
use rocket::serde::json::Json;
use rocket_dyn_templates::{Template, context};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::backenddb::game::{GamesGraph, graph_game};
use crate::frontend::ApiResult;
use crate::{SETTINGS, auth, sexymac};

#[derive(FromForm, Debug, Deserialize, JsonSchema)]
pub struct GraphForm {
    event: Option<String>,
    teams: Vec<i32>
}


#[rocket_okapi::openapi]
#[post("/api/graph_sub", data = "<body>")]
pub async fn graph(body: Json<GraphForm>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<Vec<Vec<GamesGraph>>>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    let mut result: Vec<Vec<GamesGraph>> = Vec::with_capacity(body.teams.len());
    for team in &body.teams {
        let data = match graph_game(team, &body.event, db).await {
            Ok(a) => {a},
            Err(a) => {
                return Json(ApiResult::Error(a.to_string()));
            }
        };
        result.push(data);
    }

    Json(ApiResult::Success(result))
}

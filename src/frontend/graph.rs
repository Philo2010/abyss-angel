
use rocket::State;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::post;
use rocket::serde::json::Json;
use rocket_dyn_templates::{Template, context};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::backenddb::game::{GamesGraph, graph_game};
use crate::frontend::ApiResult;
use crate::snowgrave::datatypes::TeamData;
use crate::{SETTINGS, auth, sexymac};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GraphForm {
    event: Option<String>,
    teams: Vec<TeamData>
}

#[derive(Serialize, JsonSchema, Deserialize)]
pub struct GraphTeam {
    data: Vec<GamesGraph>,
    team: TeamData,
}


#[rocket_okapi::openapi]
#[post("/api/graph_sub", data = "<body>")]
pub async fn graph(body: Json<GraphForm>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<Vec<GraphTeam>>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    let mut result: Vec<GraphTeam> = Vec::with_capacity(body.teams.len());
    for team in &body.teams {
        let data = match graph_game(&team.team, &team.is_ab_team, &body.event, db).await {
            Ok(a) => {a},
            Err(a) => {
                return Json(ApiResult::Error(a.to_string()));
            }
        };
        result.push(GraphTeam { data, team: *team });
    }

    Json(ApiResult::Success(result))
}

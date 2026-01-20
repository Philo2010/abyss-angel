use rocket::{State, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{entity::{sea_orm_active_enums::{Stations, TournamentLevels}, upcoming_game, upcoming_team}, frontend::ApiResult};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Game {
    pub id: i32,
    pub event_code: String,
    pub match_id: i32,
    pub set: i32,
    pub tournament_level: TournamentLevels,
    pub team: Vec<Team>
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Team {
    pub id: i32,
    pub station: Stations,
    pub team: i32,
    pub is_ab_team: bool,
}


#[rocket_okapi::openapi]
#[get("/api/snowgrave/all")]
pub async fn get_all_snowgrave(db: &State<DatabaseConnection>) -> Json<ApiResult<Vec<Game>>> {

    let games = match upcoming_game::Entity::find().all(db.inner()).await {
        Ok(a) => a,
        Err(a) => {
            return Json(ApiResult::Error(format!("Db Error: {a}")));
        },
    };
    let mut games_to_send: Vec<Game> = Vec::new();
    for game in games {
        let team: Vec<Team> = match upcoming_team::Entity::find()
            .filter(upcoming_team::Column::GameId.eq(game.id)).all(db.inner()).await {
                Ok(a) => a,
                Err(a) => {
                    return Json(ApiResult::Error(format!("Db Error: {a}")));
                },
            }.into_iter().map(|x| Team { id: x.id, station: x.station, team: x.team, is_ab_team: x.is_ab_team }).collect();
        games_to_send.push(Game { id: game.id, event_code: game.event_code, match_id: game.match_id, set: game.set, tournament_level: game.tournament_level, team });
    }

    Json(ApiResult::Success(games_to_send))
}
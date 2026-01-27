use rocket::{State, form::validate::eq, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{entity::{sea_orm_active_enums::Stations, upcoming_game, upcoming_team}, frontend::ApiResult, snowgrave::datatypes::TeamData};


#[derive(Serialize, JsonSchema, Deserialize)]
pub struct TeamWithAlliance {
    is_blue: bool,
    team: TeamData
}


#[rocket_okapi::openapi]
#[get("/api/pit/get_teams_from_game/<id_upcoming_game>")]
pub async fn get_teams_from_game(id_upcoming_game: i32, db: &State<DatabaseConnection>) -> Json<ApiResult<Vec<TeamWithAlliance>>> {
    let game_data = match upcoming_team::Entity::find()
        .filter(upcoming_team::Column::GameId.eq(id_upcoming_game)).all(db.inner()).await {
            Ok(a) => a,
            Err(a) => {
                return Json(ApiResult::Error(format!("Err while getting teams: {a}")));
            },
        };
    
    if game_data.is_empty() {
        return Json(ApiResult::Error("That game does not exist!".to_string()));
    }
    let team_datas: Vec<TeamWithAlliance> = game_data.into_iter().map(|x| {
        let is_blue: bool;
        if x.station == Stations::Blue1 || x.station == Stations::Blue2 || x.station == Stations::Blue3 {
            is_blue = true;
        } else {
            is_blue = false;
        }
        TeamWithAlliance { is_blue,
            team: TeamData {
                is_ab_team: x.is_ab_team,
                team: x.team
    }}
    }).collect();


    Json(ApiResult::Success(team_datas))
}
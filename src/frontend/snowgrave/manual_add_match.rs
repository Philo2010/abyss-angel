use rocket::State;
use rocket::serde::json::Json;
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::get_by_user::get_by_username;
use crate::frontend::ApiResult;
use crate::{entity::sea_orm_active_enums::TournamentLevels, frontend::snowgrave::get_all_scouts::Team};
use crate::snowgrave::manual_add_match::{ManualInsertMatch, ManualInsertTeam, MvpInsertMatch};
use crate::snowgrave::datatypes::Team as DataTeam;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ManualInsertMatchFront {
    pub event_code: String,
    pub match_id: i32,
    pub set: i32,
    pub tournament_level: TournamentLevels,
    pub mvps: MvpInsertMatchFront,
    pub teams_red: [ManualInsertTeamFront; 3],
    pub teams_blue: [ManualInsertTeamFront; 3]
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MvpInsertMatchFront {
    pub red_username: String,
    pub blue_username: String
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ManualInsertTeamFront {
    pub team: Team,
    pub scouters: Vec<String>
}

#[rocket_okapi::openapi]
#[post("/api/snowgrave/manual_add_match", data="<data>")]
pub async fn manual_add_match(data: Json<ManualInsertMatchFront>, db: &State<DatabaseConnection>) -> Json<ApiResult<String>> {
    let mvp_red_uuid = match get_by_username(&data.mvps.red_username, db.inner()).await {
        Ok(a) => a,
        Err(_) => {
            return Json(ApiResult::Error("Red username is not valid".to_string()));
        },
    };
    let mvp_blue_uuid = match get_by_username(&data.mvps.blue_username, db.inner()).await {
        Ok(a) => a,
        Err(_) => {
            return Json(ApiResult::Error("Blue username is not valid".to_string()));
        },
    };

    let mvps = MvpInsertMatch {
        red_username: mvp_red_uuid,
        blue_username: mvp_blue_uuid,
    };

    let mut red_teams: Vec<ManualInsertTeam> = Vec::new();
    for team_front in &data.teams_red {
        let mut uuid_scouters: Vec<Uuid> = Vec::new();
        for scouter in &team_front.scouters {
            let uuid_scouter = match get_by_username(scouter, db.inner()).await {
                Ok(a) => a,
                Err(_) => {
                    return Json(ApiResult::Error(format!("No valid username for scouter '{scouter}' on red alliance")));
                },
            };
            uuid_scouters.push(uuid_scouter);
        }
        red_teams.push(ManualInsertTeam {
            team: DataTeam {
                number: team_front.team.team,
                is_ab_team: team_front.team.is_ab_team,
            },
            scouters: uuid_scouters,
        });
    }

    let mut blue_teams: Vec<ManualInsertTeam> = Vec::new();
    for team_front in &data.teams_blue {
        let mut uuid_scouters: Vec<Uuid> = Vec::new();
        for scouter in &team_front.scouters {
            let uuid_scouter = match get_by_username(scouter, db.inner()).await {
                Ok(a) => a,
                Err(_) => {
                    return Json(ApiResult::Error(format!("No valid username for scouter '{scouter}' on blue alliance")));
                },
            };
            uuid_scouters.push(uuid_scouter);
        }
        blue_teams.push(ManualInsertTeam {
            team: DataTeam {
                number: team_front.team.team,
                is_ab_team: team_front.team.is_ab_team,
            },
            scouters: uuid_scouters,
        });
    }

    let teams_red: [ManualInsertTeam; 3] = match red_teams.try_into() {
        Ok(a) => a,
        Err(_) => return Json(ApiResult::Error("Expected exactly 3 red teams".to_string())),
    };
    let teams_blue: [ManualInsertTeam; 3] = match blue_teams.try_into() {
        Ok(a) => a,
        Err(_) => return Json(ApiResult::Error("Expected exactly 3 blue teams".to_string())),
    };

    let match_insert = ManualInsertMatch {
        event_code: data.event_code.clone(),
        match_id: data.match_id,
        set: data.set,
        tournament_level: data.tournament_level,
        mvps,
        teams_red,
        teams_blue,
    };

    match crate::snowgrave::manual_add_match::manual_add_match(match_insert, db.inner()).await {
        Ok(_) => Json(ApiResult::Success("Match added successfully".to_string())),
        Err(e) => Json(ApiResult::Error(format!("Database error: {e}"))),
    }
}

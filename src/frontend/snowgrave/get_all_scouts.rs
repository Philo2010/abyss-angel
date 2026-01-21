use rocket::{State, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Related};
use serde::{Deserialize, Serialize};

use crate::{auth::get_by_user::{get_by_username, get_by_uuid}, entity::{game_scouts, mvp_scouters, sea_orm_active_enums::{Stations, TournamentLevels}, upcoming_game, upcoming_team}, frontend::ApiResult};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Game {
    pub id: i32,
    pub event_code: String,
    pub match_id: i32,
    pub set: i32,
    pub tournament_level: TournamentLevels,
    pub teams: Vec<Team>,
    pub mvp_blue: Option<String>,
    pub mvp_red: Option<String>
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Team {
    pub id: i32,
    pub station: Stations,
    pub team: i32,
    pub is_ab_team: bool,
    pub scouters: Vec<String>
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
        let teams= match upcoming_team::Entity::find()
            .filter(upcoming_team::Column::GameId.eq(game.id)).all(db.inner()).await {
                Ok(a) => a,
                Err(a) => {
                    return Json(ApiResult::Error(format!("Db Error: {a}")));
                },
            };
        let mut team_format: Vec<Team> = Vec::new();
        for team in teams {
            let scouts = match game_scouts::Entity::find()
                .filter(game_scouts::Column::TeamId.eq(team.id)).all(db.inner()).await {
                    Ok(a) => a,
                    Err(a) => {
                        return Json(ApiResult::Error(format!("Db error while trying to get scouters: {a}")));
                    },
                };
            let mut scouter_usernames: Vec<String> = Vec::new();
            for scout in scouts {
                let username = match get_by_uuid(&scout.scouter_id, db).await {
                    Ok(a) => {a},
                    Err(a) => {
                        match a {
                            crate::auth::get_by_user::AuthGetUuidError::UserIsNotHere => {
                                return Json(ApiResult::Error("Could not find a user's username, this is a critcal error and should not happen".to_string()));
                            },
                            crate::auth::get_by_user::AuthGetUuidError::DatabaseError(db_err) => {
                                return Json(ApiResult::Error(format!("Could not find a user's username, this is a critcal error and should not happen: DB ERROR: {db_err}")));
                            },
                        }
                    },
                };
                scouter_usernames.push(username);
            }
            team_format.push(Team {
                id: team.id,
                station: team.station,
                team: team.team,
                is_ab_team: team.is_ab_team,
                scouters: scouter_usernames
            });
        }
        
        //Code to fetch the mvp scouters
        let red = if let Some(id) = game.mvp_id_red {
            let res = match mvp_scouters::Entity::find_by_id(id).one(db.inner()).await {
                Ok(a) => a,
                Err(a) => {
                    return Json(ApiResult::Error("Failed to find red mvp".to_string()));
                },
            };

            match res {
                Some(a) => {
                    let username = match get_by_uuid(&a.scouter, db).await {
                        Ok(a) => a,
                        Err(a) => {
                            match a {
                                crate::auth::get_by_user::AuthGetUuidError::UserIsNotHere => {
                                    return Json(ApiResult::Error("Could not find the red mvp username!".to_string()));
                                },
                                crate::auth::get_by_user::AuthGetUuidError::DatabaseError(db_err) => {
                                    return Json(ApiResult::Error(format!("Could not find the red mvp username: {db_err}")));
                                },
                            }
                        },
                    };

                    Some(username)
                },
                None => {
                    None
                }
            }
        } else {
            None
        };

        let blue = if let Some(id) = game.mvp_id_blue {
            let res = match mvp_scouters::Entity::find_by_id(id).one(db.inner()).await {
                Ok(a) => a,
                Err(a) => {
                    return Json(ApiResult::Error("Failed to find blue mvp".to_string()));
                },
            };

            match res {
                Some(a) => {
                    let username = match get_by_uuid(&a.scouter, db).await {
                        Ok(a) => a,
                        Err(a) => {
                            match a {
                                crate::auth::get_by_user::AuthGetUuidError::UserIsNotHere => {
                                    return Json(ApiResult::Error("Could not find the blue mvp username!".to_string()));
                                },
                                crate::auth::get_by_user::AuthGetUuidError::DatabaseError(db_err) => {
                                    return Json(ApiResult::Error(format!("Could not find the blue mvp username: {db_err}")));
                                },
                            }
                        },
                    };

                    Some(username)
                },
                None => {
                    None
                }
            }
        } else {
            None
        };

        games_to_send.push(Game {
            id: game.id,
            event_code: game.event_code,
            match_id: game.match_id,
            set: game.set,
            tournament_level: game.tournament_level,
            teams: team_format,
            mvp_red: red,
            mvp_blue: blue});
    }

    Json(ApiResult::Success(games_to_send))
}
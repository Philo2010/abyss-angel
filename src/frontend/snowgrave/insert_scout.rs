use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use rocket::State;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use rocket::serde::json::Json;
use uuid::Uuid;
use crate::{auth::get_by_user::get_by_username, frontend::ApiResult, snowgrave::snowgrave_insert_scouters::{GameTeamDataMvp, GameTeamDataScouter, ScouterInsertForm, insert_scouters}};
/*

struct ScouterInsertForm {
    player_indexs: Vec<Uuid>,
    //id is a ref to a team, Uuid is the MVP scouter
    matches: Vec<(i32, Vec<GameTeamDataScouter>, GameTeamDataMvp)>,
}
struct GameTeamDataScouter {
    id: usize,
    station: Stations, 
}
struct GameTeamDataMvp {
    red: Uuid,
    blue: Uuid
}*/

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct MatchData {
    upcoming_match_id: i32,
    game_scouters: Vec<GameTeamDataScouter>,
    mvp: GameTeamDataMvpFront
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct GameTeamDataMvpFront {
    red: String,
    blue: String
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ScouterInsertFront {
    player_indexs: Vec<String>,
    matches: Vec<MatchData>
}

#[rocket_okapi::openapi]
#[post("/api/snowgrave/insert_scout", data="<data>")]
pub async fn insert_scout(db: &State<DatabaseConnection>, data: Json<ScouterInsertFront>) -> Json<ApiResult<String>> {
    let data_db: ScouterInsertForm;
    let mut player_index_uuid: Vec<Uuid> = Vec::with_capacity(data.0.player_indexs.len());
    for player in data.0.player_indexs {
        let player_uuid = match get_by_username(&player, db).await {
            Ok(a) => a,
            Err(a) => {
                match a {
                    crate::auth::get_by_user::AuthGetUuidError::UserIsNotHere => {
                        return Json(ApiResult::Error(format!("Failed to get user: {player}")));
                    },
                    crate::auth::get_by_user::AuthGetUuidError::DatabaseError(db_err) => {
                        return Json(ApiResult::Error(format!("Db error while getting user {player}: {db_err}")));
                    },
                }
            },
        };
        player_index_uuid.push(player_uuid);
    }
    let mut matches_db: Vec<(i32, Vec<GameTeamDataScouter>, GameTeamDataMvp)> = Vec::new();

    for match_send in data.0.matches {
        //get mvp
        let red = match get_by_username(&match_send.mvp.red, db).await {
            Ok(a) => a,
            Err(a) => {
                match a {
                    crate::auth::get_by_user::AuthGetUuidError::UserIsNotHere => {
                        return Json(ApiResult::Error(format!("Failed to get user")));
                    },
                    crate::auth::get_by_user::AuthGetUuidError::DatabaseError(db_err) => {
                        return Json(ApiResult::Error(format!("Db error while getting user: {db_err}")));
                    },
                }
            },
        };
        let blue = match get_by_username(&match_send.mvp.blue, db).await {
            Ok(a) => a,
            Err(a) => {
                match a {
                    crate::auth::get_by_user::AuthGetUuidError::UserIsNotHere => {
                        return Json(ApiResult::Error(format!("Failed to get user")));
                    },
                    crate::auth::get_by_user::AuthGetUuidError::DatabaseError(db_err) => {
                        return Json(ApiResult::Error(format!("Db error while getting user: {db_err}")));
                    },
                }
            },
        };

        matches_db.push((match_send.upcoming_match_id, match_send.game_scouters, GameTeamDataMvp { red, blue }));
    }

    //mvp uuid
    match insert_scouters(ScouterInsertForm { player_indexs: player_index_uuid, matches: matches_db }, db).await {
        Ok(a) => a,
        Err(a) => {
            return Json(ApiResult::Error(format!("Database error: {a}")));
        },
    }


    Json(ApiResult::Success("Done!".to_string()))
}
use std::collections::HashMap;
use std::collections::HashSet;

use schemars::{JsonSchema};
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::TransactionTrait;
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, DatabaseConnection, DbErr, EntityTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity::{game_scouts, mvp_scouters, upcoming_game, upcoming_team};


pub struct ScouterInsertForm {
    pub player_indexs: Vec<Uuid>,
    //id is a ref to a team, Uuid is the MVP scouter
    pub matches: Vec<(i32, Vec<GameTeamDataScouter>, GameTeamDataMvp)>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct GameTeamDataScouter {
    index: usize,
}

pub struct GameTeamDataMvp {
    pub red: Uuid,
    pub blue: Uuid
}


pub async fn insert_scouters(
    form: ScouterInsertForm,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let txn = db.begin().await?;
    let mut scouters: Vec<game_scouts::ActiveModel> = Vec::new();
    let mut mvps: Vec<(i32, mvp_scouters::ActiveModel, mvp_scouters::ActiveModel)> = Vec::new();
    let mut teams: HashMap<i32, upcoming_team::Model> = HashMap::new();

    for matche in &form.matches {
        let team = if let Some(team) = teams.get(&matche.0) {
            team
        } else {
            let model = upcoming_team::Entity::find_by_id(matche.0)
                .one(&txn)
                .await?
                .ok_or(DbErr::RecordNotFound("No team found".to_string()))?;
            teams.insert(matche.0, model);
            teams.get(&matche.0).unwrap()
        };

        // Only touch game_scouts if scouters actually changed for this team.
        // An empty scouter list means this entry exists solely to carry an MVP
        // change — skip the delete/reinsert so we don't disturb in-progress state.
        if !matche.1.is_empty() {
            // Check if any scouts for this team have already submitted data
            let existing_scouts = game_scouts::Entity::find()
                .filter(game_scouts::Column::TeamId.eq(team.id))
                .all(&txn)
                .await?;

            let has_submitted = existing_scouts.iter().any(|s| s.game_midway.is_some());
            if has_submitted {
                return Err(DbErr::Custom(format!(
                    "Cannot reassign scouters for team {} — scouts have already submitted data for this game",
                    team.id
                )));
            }

            // Delete existing scouters for this team before re-inserting
            game_scouts::Entity::delete_many()
                .filter(game_scouts::Column::TeamId.eq(team.id))
                .exec(&txn)
                .await?;

            let mut scouter_check = HashSet::new();
            for scouter in &matche.1 {
                let scouter_id = form
                    .player_indexs
                    .get(scouter.index)
                    .ok_or(DbErr::Custom("Invalid index for players".to_string()))?;

                if !scouter_check.insert(*scouter_id) {
                    return Err(DbErr::Custom(
                        "Cannot have more than one of the same scouter on a team".to_string(),
                    ));
                }

                scouters.push(game_scouts::ActiveModel {
                    id: NotSet,
                    game_id: Set(team.game_id),
                    team_id: Set(team.id),
                    scouter_id: Set(*scouter_id),
                    done: Set(false),
                    station: Set(team.station),
                    is_redo: Set(false),
                    game_midway: Set(None),
                });
            }
        }

        let mvp_red = mvp_scouters::ActiveModel {
            id: NotSet,
            scouter: Set(matche.2.red),
            is_blue: Set(false),
            data: Set(None),
        };
        let mvp_blue = mvp_scouters::ActiveModel {
            id: NotSet,
            scouter: Set(matche.2.blue),
            is_blue: Set(true),
            data: Set(None),
        };
        mvps.push((team.game_id, mvp_red, mvp_blue));
    }

    // Insert scouts
    if !scouters.is_empty() {
        game_scouts::Entity::insert_many(scouters)
            .exec(&txn)
            .await?;
    }

    // Delete old MVPs and insert new ones per game
    // Deduplicate by game_id so we don't double-delete/insert if multiple teams from same game are in the diff
    let mut seen_games: HashSet<i32> = HashSet::new();
    for (game_id, red, blue) in mvps {
        if !seen_games.insert(game_id) {
            continue; // already handled this game
        }

        // Find and delete old MVP records via the game's existing references
        let game = upcoming_game::Entity::find_by_id(game_id)
            .one(&txn)
            .await?
            .ok_or(DbErr::Custom("Could not find game".to_string()))?;

        if let Some(old_red_id) = game.mvp_id_red {
            let old_red = mvp_scouters::Entity::find_by_id(old_red_id).one(&txn).await?;
            if let Some(ref m) = old_red {
                if m.data.is_some() {
                    return Err(DbErr::Custom("Cannot reassign MVP — red MVP has already submitted data".to_string()));
                }
            }
            mvp_scouters::Entity::delete_by_id(old_red_id)
                .exec(&txn)
                .await?;
        }
        if let Some(old_blue_id) = game.mvp_id_blue {
            let old_blue = mvp_scouters::Entity::find_by_id(old_blue_id).one(&txn).await?;
            if let Some(ref m) = old_blue {
                if m.data.is_some() {
                    return Err(DbErr::Custom("Cannot reassign MVP — blue MVP has already submitted data".to_string()));
                }
            }
            mvp_scouters::Entity::delete_by_id(old_blue_id)
                .exec(&txn)
                .await?;
        }

        let red = red.insert(&txn).await?;
        let blue = blue.insert(&txn).await?;

        let mut game_active: upcoming_game::ActiveModel = game.into();
        game_active.mvp_id_red = Set(Some(red.id));
        game_active.mvp_id_blue = Set(Some(blue.id));
        game_active.update(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}
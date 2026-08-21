use chrono::Local;
use schemars::{JsonSchema};
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{SETTINGS, backenddb::game::{DefenceTarget, GamesEditSpecific, game_dispatch}, entity::{game_scouts, scout_game_midway_insert::{self, ActiveModel}, upcoming_game}, snowgrave::check_system::check};



#[derive(Serialize, Deserialize, JsonSchema)]
pub struct EditSnow {
    pub snowgrave_scout_id: i32,
    pub defence: Option<f32>,
    pub comment: Option<String>,
    pub game: GamesEditSpecific,
    pub defence_main: Option<bool>,
    pub defence_target: Option<DefenceTarget>,
    pub auto_time: Option<f32>,
    pub dead: Option<bool>,
    pub dnf: Option<bool>,
}

pub async fn edit_scouter(data: EditSnow, db: &DatabaseConnection) -> Result<(), DbErr> {
    let snowgrave_scout = game_scouts::Entity::find_by_id(data.snowgrave_scout_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find user!".to_string()))?;
    if snowgrave_scout.done {
        return Err(DbErr::Custom("Already done!".to_string()));
    }
    if !snowgrave_scout.is_redo {
        return Err(DbErr::Custom("Please scout normaly!".to_string()));
    }
    let midway_id = snowgrave_scout.game_midway.ok_or_else(|| DbErr::Custom("Scout marked for redo but has no midway data — contact admin".to_string()))?;
    let game_data: scout_game_midway_insert::Model = scout_game_midway_insert::Entity::find_by_id(midway_id).one(db).await?.ok_or_else(|| DbErr::Custom("Midway record not found".to_string()))?;

    //asume done for now
    let mut snowgrave_scout_active: game_scouts::ActiveModel = snowgrave_scout.clone().into();
    snowgrave_scout_active.done = Set(true);
    snowgrave_scout_active.is_redo = Set(false);
    snowgrave_scout_active.update(db).await?;

    //preform the edit
    let game_funcion= game_dispatch(SETTINGS.year);
    let res = game_funcion.edit(game_data.game_id, data.game, db).await?;
    

    let game_insert: scout_game_midway_insert::ActiveModel = ActiveModel {
        id: Set(game_data.id),
        user: NotSet,
        team: NotSet,
        is_ab_team: NotSet,
        match_id: NotSet,
        set: NotSet,
        total_score: Set(res.total_score as i32),
        teleop_score: Set(res.teleop_score as i32),
        auto_score: Set(res.auto_score as i32),
        defence: data.defence.map(|x| x as i32).map(Set).unwrap_or(NotSet),
        comment: data.comment.map(Set).unwrap_or(NotSet),
        event_code: NotSet,
        tournament_level: NotSet,
        station: NotSet,
        created_at: Set(Local::now()),
        game_type_id: Set(res.game_type),
        game_id: Set(res.game_id),
        defence_main: data.defence_main.map(Set).unwrap_or(NotSet),
        defence_target: data.defence_target.map(|dt| Set(Some(dt))).unwrap_or(NotSet),
        auto_time: data.auto_time.map(Set).unwrap_or(NotSet),
        dead: data.dead.map(Set).unwrap_or(NotSet),
        dnf: data.dnf.map(Set).unwrap_or(NotSet),
    };

    game_insert.update(db).await?;
    let snowgrave_game = match upcoming_game::Entity::find_by_id(snowgrave_scout.game_id).one(db).await? {
        Some(a) => a,
        None => {
            panic!("INVAID GAME ID!!");
        },
    };
    
    //do the update

    //All of the header data should be fine so no need to edit it

    //now time to do the check itself
    check(snowgrave_scout.game_id, db).await?;
    Ok(())
}
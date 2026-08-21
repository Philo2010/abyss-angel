use chrono::Local;
use schemars::{JsonSchema};
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, DatabaseConnection, DbErr, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::{SETTINGS, auth::get_by_user::get_by_uuid, backenddb::game::{DefenceTarget, GamesInsertsSpecific, game_dispatch}, entity::{game_scouts, scout_game_midway_insert, upcoming_game, upcoming_team}, snowgrave::check_system::check};


#[derive(JsonSchema, Serialize, Deserialize)]
pub struct InsertSnow {
    pub snowgrave_scout_id: i32,
    pub game: GamesInsertsSpecific,
    pub defence: i32,
    pub comment: String,
    pub defence_main: bool,
    pub defence_target: Option<DefenceTarget>,
    pub auto_time: f32,
    pub dead: bool,
    pub dnf: bool,
}


pub async fn insert_scout(db: &DatabaseConnection, data: InsertSnow) -> Result<(), DbErr> {
    let snowgrave_scout = game_scouts::Entity::find_by_id(data.snowgrave_scout_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find user!".to_string()))?;
    if snowgrave_scout.done {
        return Err(DbErr::Custom("Already done!".to_string()));
    }
    if snowgrave_scout.is_redo {
        return Err(DbErr::Custom("Please edit not insert!".to_string()));
    }
    let snowgrave_team = upcoming_team::Entity::find_by_id(snowgrave_scout.team_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find team!".to_string()))?;
    let snowgrave_game = upcoming_game::Entity::find_by_id(snowgrave_scout.game_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find game!".to_string()))?;
    let username = match get_by_uuid(&snowgrave_scout.scouter_id, db).await {
        Ok(a) => a,
        Err(a) => match a {
            crate::auth::get_by_user::AuthGetUuidError::UserIsNotHere => {return Err(DbErr::RecordNotFound("Could not find username!".to_string()))},
            crate::auth::get_by_user::AuthGetUuidError::DatabaseError(db_err) => {return Err(db_err)},
        },
    };
    let game_funcion = game_dispatch(SETTINGS.year);

    let res = game_funcion.insert(&data.game, db).await?;
    
    let mid = scout_game_midway_insert::ActiveModel {
        id: NotSet,
        user: Set(snowgrave_scout.scouter_id),
        team: Set(snowgrave_team.team),
        is_ab_team: Set(snowgrave_team.is_ab_team),
        match_id: Set(snowgrave_game.match_id),
        set: Set(snowgrave_game.set),
        defence: Set(data.defence),
        comment: Set(data.comment),
        event_code: Set(snowgrave_game.event_code),
        tournament_level: Set(snowgrave_game.tournament_level),
        station: Set(snowgrave_team.station),
        created_at: Set(Local::now()),
        game_type_id: Set(res.game_type),
        game_id: Set(res.game_id),
        total_score: Set(res.total_score as i32),
        teleop_score: Set(res.teleop_score as i32),
        auto_score: Set(res.auto_score as i32),
        defence_main: Set(data.defence_main),
        defence_target: Set(data.defence_target),
        auto_time: Set(data.auto_time),
        dead: Set(data.dead),
        dnf: Set(data.dnf),
    };

    let game_insert = mid.insert(db).await?;
    let upcoming_game_id = snowgrave_scout.game_id;
    let mut snowgrave_scout_active: game_scouts::ActiveModel = snowgrave_scout.into();
    snowgrave_scout_active.game_midway = Set(Some(game_insert.id));
    snowgrave_scout_active.done = Set(true);
    snowgrave_scout_active.update(db).await?;

    check(upcoming_game_id, db).await?;

    Ok(())
}
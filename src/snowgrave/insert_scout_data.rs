use chrono::Local;
use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, DatabaseConnection, DbErr, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::{SETTINGS, auth::get_by_user::get_by_uuid, backenddb::{self, game::{GamesInserts, GamesInsertsSpecific, HeaderInsert, game_dispatch}}, entity::{game_scouts, mvp_data, mvp_scouters, scout_game_midway_insert, sea_orm_active_enums::Stations, upcoming_game, upcoming_team}, snowgrave::check_complete::{self, CheckMatchErr}};


#[derive(JsonSchema, Serialize, Deserialize)]
pub struct InsertSnow {
    //Id is given by server
    //pub user: String, //We will get uuid
    //pub team: i32,
    //pub is_ab_team: bool,
    //pub match_id: i32,
    //pub set: i32,
    //Total score is irraiven as it will be computed at server side
    //pub event_code: String,
    //pub tournament_level: TournamentLevels,
    //pub station: Stations,
    pub snowgrave_scout_id: i32,
    //Created At no need to import as this will be seen by the server
    //game_type_id polymorfism will be seen by the enum
    //No need for game id as that will be seen by the enum
    pub game: GamesInsertsSpecific,
    pub defence: i32,
    pub comment: String,
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
    };

    let game_insert = mid.insert(db).await?;
    let mut snowgrave_scout_active: game_scouts::ActiveModel = snowgrave_scout.into();
    snowgrave_scout_active.game_midway = Set(Some(game_insert.id));
    snowgrave_scout_active.update(db).await?;

    let res = if let Err(CheckMatchErr::DbErr(a)) = check_complete::check_match(snowgrave_game.id, db).await {
        return Err(a);
    };

    Ok(())
}
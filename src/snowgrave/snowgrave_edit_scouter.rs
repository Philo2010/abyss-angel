use chrono::Local;
use schemars::{JsonSchema};
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{SETTINGS, backenddb::{self, game::{GamesEdit, GamesEditSpecific, HeaderFullEdit, game_dispatch}}, entity::{game_scouts, genertic_header, scout_game_midway_insert::{self, ActiveModel}, upcoming_game}, snowgrave::check_system::check};



#[derive(Serialize, Deserialize, JsonSchema)]
pub struct EditSnow {
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
    pub defence: Option<f32>,
    pub comment: Option<String>,
    //Created At no need to import as this will be seen by the server
    //game_type_id polymorfism will be seen by the enum
    //No need for game id as that will be seen by the enum
    pub game: GamesEditSpecific,
}

pub async fn edit_scouter(data: EditSnow, db: &DatabaseConnection) -> Result<(), DbErr> {
    let snowgrave_scout = game_scouts::Entity::find_by_id(data.snowgrave_scout_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find user!".to_string()))?;
    if snowgrave_scout.done {
        return Err(DbErr::Custom("Already done!".to_string()));
    }
    if !snowgrave_scout.is_redo {
        return Err(DbErr::Custom("Please scout normaly!".to_string()));
    }
    let game_data: scout_game_midway_insert::Model = scout_game_midway_insert::Entity::find_by_id(snowgrave_scout.game_midway.unwrap()).one(db).await?.unwrap();

    //asume done for now
    let mut snowgrave_scout_active: game_scouts::ActiveModel = snowgrave_scout.clone().into();
    snowgrave_scout_active.done = Set(true);
    snowgrave_scout_active.is_redo = Set(false);
    snowgrave_scout_active.update(db).await?;

    //preform the edit
    let game_funcion= game_dispatch(SETTINGS.year);
    let res = game_funcion.edit(game_data.game_id, data.game, db).await?;
    

    let game_insert: scout_game_midway_insert::ActiveModel = ActiveModel {
        id: NotSet,
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
    check(game_data.id, db).await?;
    Ok(())
}
use rocket::data::N;
use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{backenddb::{self, game::{GamesEdit, GamesEditSpecific, HeaderFullEdit}}, entity::{game_scouts, genertic_header, upcoming_game, upcoming_team}, snowgrave::check_complete};



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
    let snowgrave_game = upcoming_game::Entity::find_by_id(snowgrave_scout.game_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find game!".to_string()))?;
    let game_data = genertic_header::Entity::find()
        .filter(genertic_header::Column::SnowgraveScoutId.eq(data.snowgrave_scout_id)).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find game data!".to_string()))?;

    //asume done for now
    let mut snowgrave_scout_active: game_scouts::ActiveModel = snowgrave_scout.clone().into();
    snowgrave_scout_active.done = Set(true);
    snowgrave_scout_active.is_redo = Set(false);
    snowgrave_scout_active.update(db).await?;
    
    //do the update

    //All of the header data should be fine so no need to edit it
    let header = HeaderFullEdit {
        id: game_data.id,
        user: None,
        team: None,
        is_ab_team: None,
        match_id: None,
        set: None,
        event_code: None,
        tournament_level: None,
        station: None,
        created_at: None,
        is_marked: Some(false),
        is_pending: Some(true),
        snowgrave_id: None,
        is_mvp: None,
    };
    let res = backenddb::game::edit_game(GamesEdit {
        header,
        game: data.game
    }, db).await?;

    //now time to do the check itself
    let res = check_complete::check_match(snowgrave_game.id, db).await;

    todo!()
}
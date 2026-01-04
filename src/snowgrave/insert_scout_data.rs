use sea_orm::{DatabaseConnection, DbErr, EntityTrait, ModelTrait};

use crate::{auth::get_by_user::get_by_uuid, backenddb::{self, game::{GamesInserts, GamesInsertsSpecific, HeaderInsert}}, entity::{game_scouts, mvp_data, mvp_scouters, upcoming_game, upcoming_team}};

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
}


pub async fn insert_scout(db: &DatabaseConnection, data: InsertSnow) -> Result<(), DbErr> {
    let snowgrave_scout = game_scouts::Entity::find_by_id(data.snowgrave_scout_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find user!".to_string()))?;
    let snowgrave_team = upcoming_team::Entity::find_by_id(snowgrave_scout.team_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find team!".to_string()))?;
    let snowgrave_game = upcoming_game::Entity::find_by_id(snowgrave_scout.game_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find game!".to_string()))?;
    let username = match get_by_uuid(&snowgrave_scout.scouter_id, db).await {
        Ok(a) => a,
        Err(a) => match a {
            crate::auth::get_by_user::AuthGetUuidError::UserIsNotHere => {return Err(DbErr::RecordNotFound("Could not find username!".to_string()))},
            crate::auth::get_by_user::AuthGetUuidError::DatabaseError(db_err) => {return Err(db_err)},
        },
    };
    let mut is_mvp = false;
    if let Some(mvp_id) = snowgrave_game.mvp_id {
        let mvp_scouters = mvp_scouters::Entity::find_by_id(mvp_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find mvp!".to_string()))?;
        if let Some(mvp_data_id) = mvp_scouters.data {
            let mvp_data = mvp_data::Entity::find_by_id(mvp_data_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find mvp data!".to_string()))?;
            if snowgrave_team.team == mvp_data.mvp_team && snowgrave_team.is_ab_team == mvp_data.mvp_is_ab_team {
                is_mvp = true;
            }
        }
    }

    let header = HeaderInsert {
        user: username,
        team: snowgrave_team.team,
        is_ab_team: snowgrave_team.is_ab_team,
        match_id: snowgrave_game.match_id,
        set: snowgrave_game.set,
        event_code: snowgrave_game.event_code,
        tournament_level: snowgrave_game.tournament_level,
        station: snowgrave_team.station,
        snowgrave_scout_id: data.snowgrave_scout_id,
        is_mvp
    };
    let insert_stuff = GamesInserts {
        header,
        game: data.game,
    };
    let _res = backenddb::game::insert_game(&insert_stuff, db).await?; // dont need id 

    Ok(())
}
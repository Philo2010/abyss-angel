use schemars::JsonSchema;
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::dynamic::Column;
use sea_orm::sea_query::SelectStatement;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, QueryFilter, QuerySelect};
use sea_orm::sqlx::types::chrono::{self, DateTime, Local, TimeZone};
use sea_orm::{DbErr};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;
use crate::auth::get_by_user::{AuthGetUuidError, get_by_uuid};
use crate::backenddb::entrys::example_game::Avg;
use crate::entity::genertic_header;
use crate::entity::prelude::GenerticHeader;
use crate::{SETTINGS, auth, backenddb::*};
use crate::define_games;
use itertools::Itertools;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::entity::sea_orm_active_enums::{Stations, TournamentLevels};

async fn to_full_match(model: genertic_header::Model, db: &DatabaseConnection) -> Result<HeaderFull, DbErr> {
    let created_at = match Local.from_local_datetime(&model.created_at).single() {
        Some(a) => a,
        None => {
            return Err(DbErr::Custom("Could not parse time!".to_string()));
        },
    };
    let username = match auth::get_by_user::get_by_uuid(&model.user, db).await {
        Ok(a) => a,
        Err(a) => {
                match a {
                    AuthGetUuidError::UserIsNotHere => {
                        return Err(DbErr::Custom("User was not found".to_string()));
                    },
                    AuthGetUuidError::DatabaseError(db_err) => {
                        return Err(db_err);
                    },
                }
        },
    };
    

    Ok(HeaderFull {
        id: model.id,
        user: username,
        team: model.team,
        is_ab_team: model.is_ab_team,
        match_id: model.match_id,
        set: model.set,
        total_score: model.total_score,
        event_code: model.event_code,
        tournament_level: model.tournament_level,
        station: model.station,
        created_at: created_at,
        is_marked: model.is_marked,
        is_pending: model.is_pending,
        is_mvp: model.is_mvp,
        snowgrave_scout_id: model.snowgrave_scout_id
    })
}

#[async_trait]
pub trait YearOp: Send + Sync {
    fn get_year_id(&self) -> i32;
    async fn insert(&self, data: &GamesInsertsSpecific, db: &DatabaseConnection) -> Result<(i32, i32, i32), DbErr>;
    async fn graph(&self, ids: Vec<i32>, db: &DatabaseConnection) -> Result<Vec<GamesGraphSpecific>, DbErr>;
    // Not is the name order!
    async fn average_team(&self, ids: Vec<(i32, Vec<i32>)>, db: &DatabaseConnection) -> Result<Vec<(i32, GamesAvgSpecific)>, DbErr>;
    async fn get_full_matches(&self, ids: Vec<i32>, db: &DatabaseConnection) -> Result<Vec<GamesFullSpecific>, DbErr>;
    async fn delete(&self, id: i32, db: &DatabaseConnection) -> Result<(), DbErr>;
    async fn get(&self, id: i32, db: &DatabaseConnection) -> Result<GamesFullSpecific, DbErr>;
    async fn edit(&self, header: genertic_header::ActiveModel, edit: GamesEditSpecific, db: &DatabaseConnection) -> Result<(), DbErr>;
}


//A common header that will be used for Insert data
pub struct HeaderInsert {
    //Id is given by server
    pub user: String, //We will get uuid
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    //Total score is irraiven as it will be computed at server side
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    pub snowgrave_scout_id: i32,
    pub is_mvp: bool,
    //Created At no need to import as this will be seen by the server
    //game_type_id polymorfism will be seen by the enum
    //No need for game id as that will be seen by the enum
}

async fn prim_insert_game(data: &GamesInserts, model: Box<dyn YearOp>, db: &DatabaseConnection) -> Result<i32, DbErr> {
    //Insert game spcific
    let (game_type_id, game_id, total_score) = model.insert(&data.game, db).await?;
    
    //Get UUid
    let a = match crate::auth::get_by_user::get_by_username(&data.header.user, db).await {
        Ok(a) => a,
        Err(a) => {
            match a {
                AuthGetUuidError::UserIsNotHere => {
                    return Err(DbErr::Custom("User was not found".to_string()));
                },
                AuthGetUuidError::DatabaseError(db_err) => {
                    return Err(db_err);
                },
            }
        },
    };
    let created_at: DateTime<Local> = chrono::Local::now();


    let header_db: genertic_header::ActiveModel = genertic_header::ActiveModel {
        id: NotSet, //Done by db
        user: Set(a),
        team: Set(data.header.team),
        is_ab_team: Set(data.header.is_ab_team),
        match_id: Set(data.header.match_id),
        set: Set(data.header.set),
        total_score: Set(total_score),
        event_code: Set(data.header.event_code.clone()),
        tournament_level: Set(data.header.tournament_level.clone()),
        station: Set(data.header.station.clone()),
        created_at: Set(created_at.naive_local()),
        is_mvp: Set(data.header.is_mvp),
        game_type_id: Set(game_type_id),
        game_id: Set(game_id),
        is_marked: Set(false),
        is_pending: Set(true),
        is_dup: Set(true),
        snowgrave_scout_id: Set(data.header.snowgrave_scout_id)
    };
    Ok(genertic_header::Entity::insert(header_db).exec(db).await?.last_insert_id)
}

async fn prim_graph_game(model: Box<dyn YearOp>, team: &i32, event_code: &Option<String>, db: &DatabaseConnection) -> Result<Vec<GamesGraph>, DbErr> {
    
    let mut command = genertic_header::Entity::find()
        .filter(genertic_header::Column::Team.eq(*team))
        .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()))
        .filter(genertic_header::Column::IsMarked.eq(false))
        .filter(genertic_header::Column::IsPending.eq(false))
        .filter(genertic_header::Column::IsDup.eq(false));
    if let Some(e) = event_code {
        command = command.filter(genertic_header::Column::EventCode.eq(e));
    }
    let res: Vec<(HeaderGraph, i32)> = command
        .select_only()
        .column(genertic_header::Column::CreatedAt)
        .column(genertic_header::Column::TotalScore)
        .column(genertic_header::Column::GameId)
        .into_tuple()
        .all(db)
        .await?.iter().map(|x: &(DateTime<Local>, i32, i32)| {
        (HeaderGraph {
            time: x.0,
            total_score: x.1
        }, x.2)
    }).collect();

    let game_data = model.graph(res.iter().map(|x | x.1).collect(), db).await?;

    let header: Vec<HeaderGraph> = res.into_iter().map(|x| x.0).collect();

    let merged: Vec<GamesGraph> = header.into_iter()
        .zip(game_data.into_iter())
        .map(|(header, game)| GamesGraph { header, game}).collect();

    Ok(merged)
}

async fn prim_search_game(mode: Box<dyn YearOp>, param: &SearchParam, db: &DatabaseConnection) -> Result<Vec<GamesFull>, DbErr> {
    let mut game_headers = genertic_header::Entity::find().filter(genertic_header::Column::GameTypeId.eq(param.year));

    if let Some(user) = &param.user {
        let a = match crate::auth::get_by_user::get_by_username(user, db).await {
            Ok(a) => a,
            Err(a) => {
                match a {
                    AuthGetUuidError::UserIsNotHere => {
                        return Err(DbErr::Custom("User was not found".to_string()));
                    },
                    AuthGetUuidError::DatabaseError(db_err) => {
                        return Err(db_err);
                    },
                }
            },
        };
        game_headers = game_headers.filter(genertic_header::Column::User.eq(a));
    }
    if let Some(team) = &param.team {
        game_headers = game_headers.filter(genertic_header::Column::Team.eq(*team));
    }
    if let Some(is_ab_team) = &param.is_ab_team {
        game_headers = game_headers.filter(genertic_header::Column::IsAbTeam.eq(*is_ab_team));
    }
    if let Some(match_id) = &param.match_id {
        game_headers = game_headers.filter(genertic_header::Column::MatchId.eq(*match_id));
    }
    if let Some(set) = &param.set {
        game_headers = game_headers.filter(genertic_header::Column::Set.eq(*set));
    }
    if let Some(total_score) = &param.total_score {
        game_headers = game_headers.filter(genertic_header::Column::TotalScore.eq(*total_score));
    }
    if let Some(event_code) = &param.event_code {
        game_headers = game_headers.filter(genertic_header::Column::EventCode.eq(event_code));
    }
    if let Some(tournament_level) = &param.tournament_level {
        game_headers = game_headers.filter(genertic_header::Column::TournamentLevel.eq(tournament_level.clone()));
    }
    if let Some(station) = &param.station {
        game_headers = game_headers.filter(genertic_header::Column::EventCode.eq(station.clone()));
    }
    if let Some(mvp) = &param.is_mvp {
        game_headers = game_headers.filter(genertic_header::Column::IsMvp.eq(*mvp));
    }

    let res = game_headers
        .filter(genertic_header::Column::IsMarked.eq(false))
        .filter(genertic_header::Column::IsPending.eq(false))
        .filter(genertic_header::Column::IsDup.eq(false))
        .all(db).await?;
    let ids: Vec<i32> = res.iter().map(|a| a.game_id).collect();

    let mut header: Vec<HeaderFull> = Vec::with_capacity(res.len());
    for head in res {
        header.push(to_full_match(head, db).await?);
    }


    let games = mode.get_full_matches(ids, db).await?;

    let merged: Vec<GamesFull> = header.into_iter().zip(games.into_iter())
        .map(|x | GamesFull {header: x.0, game: x.1} ).collect();


    Ok(merged)
}


#[derive(FromQueryResult)]
struct NormalGenDataAvg {
    pub team: i32,
    pub total_score: f32,
}
#[derive(FromQueryResult)]
pub struct NormalSpcDataAvg {
    pub team: i32,
    pub game_id: i32,
}

async fn prim_average_game(model: Box<dyn YearOp>, event_code: &String, db: &DatabaseConnection) -> Result<Vec<GamesAvg>, DbErr> {
    let select_avg_score: Vec<NormalGenDataAvg> = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()))
        .filter(genertic_header::Column::EventCode.eq(event_code))
        .filter(genertic_header::Column::IsMarked.eq(false))
        .filter(genertic_header::Column::IsPending.eq(false))
        .filter(genertic_header::Column::IsDup.eq(false))
        .select_only()
        .column_as(genertic_header::Column::TotalScore.avg(), "total_score")
        .column_as(genertic_header::Column::Team, "team")
        .group_by(genertic_header::Column::Team)
        .into_model::<NormalGenDataAvg>()
        .all(db).await?;
    
    
    let ids: Vec<NormalSpcDataAvg> = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()))
        .filter(genertic_header::Column::EventCode.eq(event_code))
        .filter(genertic_header::Column::IsMarked.eq(false))
        .filter(genertic_header::Column::IsPending.eq(false))
        .filter(genertic_header::Column::IsDup.eq(false))
        .select_only()
        .column(genertic_header::Column::GameId)
        .column(genertic_header::Column::Team)
        .into_model::<NormalSpcDataAvg>().all(db).await?;

    let data: Vec<(i32, Vec<i32>)> = ids.into_iter()
        .into_group_map_by(|record| record.team)
        .into_iter()
        .map(|(team, records)| {
            (team, records.into_iter().map(|r| r.game_id).collect())
        })
        .collect();

    let avg_map: HashMap<i32, f32> = select_avg_score.into_iter().map(|x|
        (x.team, x.total_score)).collect();

    let a: Vec<(i32, GamesAvgSpecific)> = model.average_team(data, db).await?;
    let mut done: Vec<GamesAvg> = Vec::with_capacity(a.len());
    for spc in a {
        let avg = avg_map.get(&spc.0).ok_or(DbErr::AttrNotSet("Could not find avg data".to_string()))?;
        done.push(GamesAvg { team: spc.0, total_score: *avg, game: spc.1 });
    }


    
    Ok(done)
}

async fn prim_get_game(model: Box<dyn YearOp>, id: i32, db: &DatabaseConnection) -> Result<GamesFull, DbErr> {
    let header = match genertic_header::Entity::find_by_id(id).one(db).await? {
        None => {
            return Err(DbErr::RecordNotFound("Could not find".to_string()));
        },
        Some(a) => {
            a
        }
    };

    let game = model.get(header.game_id, db).await?;
    let right_header = to_full_match(header, db).await?;

    Ok(GamesFull {
        header: right_header,
        game,
    })
}

async fn prim_edit_game(model: Box<dyn YearOp>, edit: GamesEdit, db: &DatabaseConnection) -> Result<(), DbErr> {
    let game_id = to_full_am(edit.header, db).await?;

    model.edit(game_id, edit.game, db).await?;

    Ok(())
}

pub struct GamesInserts {
    pub header: HeaderInsert,
    pub game: GamesInsertsSpecific
}

#[derive(Serialize, JsonSchema)]
pub struct HeaderGraph  {
    pub time: DateTime<Local>,
    pub total_score: i32,
}

#[derive(Serialize, JsonSchema)]
pub struct GamesGraph {
    pub header: HeaderGraph,
    pub game: GamesGraphSpecific
}

#[derive(Serialize, JsonSchema)]
pub struct GamesAvg {
    pub team: i32,
    pub total_score: f32,
    pub game: GamesAvgSpecific
}

#[derive(Serialize, JsonSchema)]
pub struct GamesFull {
    pub header: HeaderFull,
    pub game: GamesFullSpecific
}

pub struct GamesEdit {
    pub header: HeaderFullEdit,
    pub game: GamesEditSpecific
}

pub struct SearchParam {
    //Id should be done via get
    pub user: Option<String>,
    pub team: Option<i32>,
    pub is_ab_team: Option<bool>,
    pub match_id: Option<i32>,
    pub set: Option<i32>,
    pub total_score: Option<i32>,
    pub event_code: Option<String>,
    pub tournament_level: Option<TournamentLevels>,
    pub is_mvp: Option<bool>,
    pub station: Option<Stations>,
    pub year: i32,
}

#[derive(Serialize, JsonSchema)]
pub struct HeaderFull {
    pub id: i32,
    pub user: String,
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    pub total_score: i32,
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    pub created_at: DateTime<Local>,
    pub is_pending: bool,
    pub is_marked: bool,
    pub is_mvp: bool,
    pub snowgrave_scout_id: i32,
}

pub struct HeaderFullEdit {
    pub id: i32,
    pub user: Option<String>,
    pub team: Option<i32>,
    pub is_ab_team: Option<bool>,
    pub match_id: Option<i32>,
    pub set: Option<i32>,
    pub event_code: Option<String>,
    pub tournament_level: Option<TournamentLevels>,
    pub station: Option<Stations>,
    pub created_at: Option<DateTime<Local>>,
    pub is_marked: Option<bool>,
    pub is_pending: Option<bool>,
    pub snowgrave_id: Option<i32>,
    pub is_mvp: Option<bool>
}

async fn to_full_am(header: HeaderFullEdit, db: &DatabaseConnection) -> Result<genertic_header::ActiveModel, DbErr> {

    let created_at;

    if let Some(c) = header.created_at {
        created_at = Some(c.naive_local());
    } else {
        created_at = None;
    }
    let username: Option<Uuid>;

    if let Some(name) = header.user {
        username = match auth::get_by_user::get_by_username(&name, db).await {
            Ok(a) => Some(a),
            Err(a) => {
                    match a {
                        AuthGetUuidError::UserIsNotHere => {
                            return Err(DbErr::Custom("User was not found".to_string()));
                        },
                        AuthGetUuidError::DatabaseError(db_err) => {
                            return Err(db_err);
                        },
                    }
            },
        };
    } else {
        username = None;
    }
    
    //get gametype and game id (for later insert into game)
    let game_model = match genertic_header::Entity::find_by_id(header.id).one(db).await? {
        Some(a) => a,
        None => {
            return Err(DbErr::Custom("Not a vaild ID".to_string()));
        },
    };
    
    Ok(genertic_header::ActiveModel {
        id: Set(header.id),
        user: username.map(Set).unwrap_or(NotSet),
        team: header.team.map(Set).unwrap_or(NotSet),
        is_ab_team: header.is_ab_team.map(Set).unwrap_or(NotSet),
        match_id: header.match_id.map(Set).unwrap_or(NotSet),
        set: header.set.map(Set).unwrap_or(NotSet),
        total_score: NotSet, //Will be set later
        event_code: header.event_code.map(Set).unwrap_or(NotSet),
        tournament_level: header.tournament_level.map(Set).unwrap_or(NotSet),
        station: header.station.map(Set).unwrap_or(NotSet),
        created_at: created_at.map(Set).unwrap_or(NotSet),
        is_marked: header.is_marked.map(Set).unwrap_or(NotSet),
        is_pending: header.is_pending.map(Set).unwrap_or(NotSet),
        is_dup: NotSet, //We never change this, that is snowgrave's job
        snowgrave_scout_id: header.snowgrave_id.map(Set).unwrap_or(NotSet),
        is_mvp: header.is_mvp.map(Set).unwrap_or(NotSet),
        game_type_id: Set(game_model.game_type_id),
        game_id: Set(game_model.game_id),
    })
}

//REAL functions that help grab the value

pub async fn insert_game(data: &GamesInserts, db: &DatabaseConnection) -> Result<i32, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_insert_game(data, game, db).await
}

pub async fn graph_game(team: &i32, event_code: &Option<String>, db: &DatabaseConnection) -> Result<Vec<GamesGraph>, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_graph_game(game, team, event_code, db).await
}

pub async fn search_game(param: &SearchParam, db: &DatabaseConnection) -> Result<Vec<GamesFull>, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_search_game(game, param, db).await
}

pub async fn average_game(event_code: &String, db: &DatabaseConnection) -> Result<Vec<GamesAvg>, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_average_game(game, event_code, db).await
}

pub async fn get_game(id: i32, db: &DatabaseConnection) -> Result<GamesFull, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_get_game(game, id, db).await
}

pub async fn delete_game(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    let game = game_dispatch(SETTINGS.year);

    game.delete(id, db).await
}

pub async fn edit_game(edit: GamesEdit, db: &DatabaseConnection) -> Result<(), DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_edit_game(game, edit, db).await
} 


define_games!(
    //Insert each year here
    ExampleGame => crate::backenddb::entrys::example_game,
);  
use schemars::JsonSchema;
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::prelude::Expr;
use sea_orm::sea_query::Alias;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, FromQueryResult, QueryFilter, QuerySelect};
use sea_orm::sqlx::types::chrono::{self, DateTime, Local};
use sea_orm::{DbErr};
use serde::Serialize;
use uuid::Uuid;
use crate::auth::get_by_user::AuthGetUuidError;
use crate::entity::genertic_header;
use crate::{SETTINGS, auth, backenddb};
use crate::define_games;
use itertools::Itertools;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::entity::sea_orm_active_enums::{Stations, TournamentLevels};

async fn to_full_match(model: genertic_header::Model, db: &DatabaseConnection) -> Result<HeaderFull, DbErr> {
    let mut username_str: Vec<String> = Vec::with_capacity(model.user.len());
    
    for user in &model.user {
        let user_str =  auth::get_by_user::get_by_uuid(user, db).await.unwrap();
        username_str.push(user_str);
    }

    Ok(HeaderFull {
        id: model.id,
        user: username_str,
        team: model.team,
        is_ab_team: model.is_ab_team,
        match_id: model.match_id,
        set: model.set,
        total_score: model.total_score,
        event_code: model.event_code,
        tournament_level: model.tournament_level,
        station: model.station,
        created_at: model.created_at,
        is_mvp: model.is_mvp,
        defence: model.defence,
        auto_score: model.auto_score,
        comment: model.comment,
        teleop_score: model.teleop_score,
    })
}

pub struct InsertReturn {
    pub game_type: i32,
    pub game_id: i32,
    pub total_score: f32,
    pub teleop_score: f32,
    pub auto_score: f32,
}

pub struct AvgReturn {
    pub team: i32,
    pub is_ab_team: bool,
    pub data: GamesAvgSpecific
}

pub struct FrontRunnerReturn {
    pub crazy: Vec<usize>,
    pub avg: GamesInserts
}

pub struct Scores {
    pub total_score: f32,
    pub teleop_score: f32,
    pub auto_score: f32,
}

#[async_trait]
pub trait YearOp: Send + Sync {
    fn get_year_id(&self) -> i32;
    async fn insert(&self, data: &GamesInsertsSpecific, db: &DatabaseConnection) -> Result<InsertReturn, DbErr>;
    // Not is the name order!
    async fn average_team(&self, ids: Vec<TeamGameUnMergedData>, db: &DatabaseConnection) -> Result<Vec<AvgReturn>, DbErr>;
    async fn get_full_matches(&self, game_ids: Vec<i32>, db: &DatabaseConnection) -> Result<Vec<GamesFullSpecific>, DbErr>;
    async fn delete(&self, id: i32, db: &DatabaseConnection) -> Result<(), DbErr>;
    #[allow(dead_code)]
    async fn get(&self, id: i32, db: &DatabaseConnection) -> Result<GamesFullSpecific, DbErr>;
    fn frontrunner_op(&self, games: &FrontRunnerGame) -> Result<FrontRunnerReturn, DbErr>;
    fn get_scores(&self, match_data: &GamesInsertsSpecific) -> Scores;
    async fn edit(&self, game_id: i32, edit: GamesEditSpecific, db: &DatabaseConnection) -> Result<InsertReturn, DbErr>;
}


//A common header that will be used for Insert data
#[derive(Debug)]
pub struct HeaderInsert {
    //Id is given by server
    pub user: Vec<Uuid>, //We will get uuid
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    //Total score is irraiven as it will be computed at server side
    pub defence: f32,
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    pub is_mvp: bool,
    pub comment: String,
    //Created At no need to import as this will be seen by the server
    //game_type_id polymorfism will be seen by the enum
    //No need for game id as that will be seen by the enum
}

async fn prim_insert_game(data: &GamesInserts, model: Box<dyn YearOp>, db: &DatabaseConnection) -> Result<i32, DbErr> {
    //Insert game spcific
    let res = model.insert(&data.game, db).await?;
    
    let created_at: DateTime<Local> = chrono::Local::now();


    let header_db: genertic_header::ActiveModel = genertic_header::ActiveModel {
        id: NotSet, //Done by db
        user: Set(data.header.user.clone()),
        team: Set(data.header.team),
        is_ab_team: Set(data.header.is_ab_team),
        match_id: Set(data.header.match_id),
        set: Set(data.header.set),
        total_score: Set(res.total_score),
        event_code: Set(data.header.event_code.clone()),
        tournament_level: Set(data.header.tournament_level),
        station: Set(data.header.station),
        created_at: Set(created_at),
        is_mvp: Set(data.header.is_mvp),
        game_type_id: Set(res.game_type),
        game_id: Set(res.game_id),
        teleop_score: Set(res.teleop_score),
        auto_score: Set(res.auto_score),
        defence: Set(data.header.defence),
        comment: Set(data.header.comment.clone()),
    };
    Ok(genertic_header::Entity::insert(header_db).exec(db).await?.last_insert_id)
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
        game_headers = game_headers.filter(genertic_header::Column::TournamentLevel.eq(*tournament_level));
    }
    if let Some(station) = &param.station {
        game_headers = game_headers.filter(genertic_header::Column::EventCode.eq(*station));
    }
    if let Some(mvp) = &param.is_mvp {
        game_headers = game_headers.filter(genertic_header::Column::IsMvp.eq(*mvp));
    }

    let res = game_headers.all(db).await?;
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
    pub is_ab_team: bool,
    pub total_score: f64,
    pub auto_score: f64,
    pub teleop_score: f64,
    pub defence_score: f64,
    pub mvp_percent: f64,
}
#[derive(FromQueryResult)]
pub struct NormalSpcDataAvg {
    pub team: i32,
    pub is_ab_team: bool,
    pub game_id: i32,
}

pub struct TeamGameUnMergedData {
    pub team: i32,
    pub is_ab_team: bool,
    pub game_ids: Vec<i32>,
}

pub struct Scores_E {
    pub total_score: f64,
    pub auto_score: f64,
    pub teleop_score: f64,
    pub defence: f64,
    pub mvp_percent: f64,
}

async fn prim_average_game(model: Box<dyn YearOp>, event_code: &String, db: &DatabaseConnection) -> Result<Vec<TeamAvg>, DbErr> {
    let select_avg_score: Vec<NormalGenDataAvg> = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()))
        .filter(genertic_header::Column::EventCode.eq(event_code))
        .select_only()
        .column_as(genertic_header::Column::TotalScore.avg().cast_as(Alias::new("FLOAT8")), "total_score")
        .column_as(genertic_header::Column::AutoScore.avg().cast_as(Alias::new("FLOAT8")), "auto_score")
        .column_as(genertic_header::Column::TeleopScore.avg().cast_as(Alias::new("FLOAT8")), "teleop_score")
        .column_as(genertic_header::Column::Defence.avg().cast_as(Alias::new("FLOAT8")), "defence_score")
        .column_as(Expr::col(genertic_header::Column::IsMvp).cast_as(Alias::new("int")).avg().cast_as(Alias::new("FLOAT8")),"mvp_percent")
        .column_as(genertic_header::Column::Team, "team")
        .column_as(genertic_header::Column::IsAbTeam, "is_ab_team")
        .group_by(genertic_header::Column::Team)
        .group_by(genertic_header::Column::IsAbTeam)
        .into_model::<NormalGenDataAvg>()
        .all(db).await?;
    
    
    let ids: Vec<NormalSpcDataAvg> = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()))
        .filter(genertic_header::Column::EventCode.eq(event_code))
        .select_only()
        .column(genertic_header::Column::GameId) //Not snowgrave
        .column(genertic_header::Column::Team)
        .column(genertic_header::Column::IsAbTeam)
        .into_model::<NormalSpcDataAvg>().all(db).await?;

    let data: Vec<TeamGameUnMergedData> = ids.into_iter()
        .into_group_map_by(|record| (record.team, record.is_ab_team))
        .into_iter()
        .map(|((team, is_ab_team), records)| {
            TeamGameUnMergedData { team, is_ab_team, game_ids: records.into_iter().map(|r| r.game_id).collect() }
        })
        .collect();

    let avg_map: HashMap<(i32, bool), Scores_E> = select_avg_score.into_iter().map(|x|
        ((x.team, x.is_ab_team), Scores_E { total_score: x.total_score,
            auto_score: x.auto_score,
            teleop_score: x.teleop_score,
            defence: x.defence_score,
            mvp_percent: x.mvp_percent,
        })).collect();

    let a: Vec<AvgReturn> = model.average_team(data, db).await?;
    let mut done: Vec<TeamAvg> = Vec::with_capacity(a.len());
    for team_avg_data in a {
        let avg = avg_map.get(&(team_avg_data.team, team_avg_data.is_ab_team)).ok_or(DbErr::AttrNotSet("Could not find avg data".to_string()))?;
        done.push(TeamAvg { team: team_avg_data.team,
            is_ab_team: team_avg_data.is_ab_team,
            total_score: avg.total_score,
            auto_score: avg.auto_score,
            teleop_score: avg.teleop_score,
            defence_score: avg.defence,
            game: team_avg_data.data,
            mvp_percent: avg.mvp_percent,
        });
    }


    
    Ok(done)
}
#[allow(dead_code)]
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


#[derive(Debug)]
pub struct GamesInserts {
    pub header: HeaderInsert,
    pub game: GamesInsertsSpecific
}


#[derive(Serialize, JsonSchema, FromQueryResult, Deserialize)]
pub struct GamesGraph {
    pub time: DateTime<Local>,
    pub total_score: i32,
    pub auto_score: i32,
    pub teleop_score: i32,
    pub defence: i32,
}

#[derive(Serialize, JsonSchema)]
pub struct TeamAvg {
    pub team: i32,
    pub is_ab_team: bool,
    pub total_score: f64,
    pub auto_score: f64,
    pub teleop_score: f64,
    pub defence_score: f64,
    pub mvp_percent: f64,
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
    pub user: Vec<String>,
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    pub total_score: f32,
    pub auto_score: f32,
    pub teleop_score: f32,
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    pub comment: String,
    pub created_at: DateTime<Local>,
    pub is_mvp: bool,
    pub defence: f32,
}

#[derive()]
pub struct FrontRunnerGame {
    pub games: Vec<(Uuid, GamesFullSpecific)>,
    pub defence: Vec<f32>,
    pub comment: Vec<String>,
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    //Total score is irraiven as it will be computed at server side
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    pub is_mvp: bool,
}

pub struct HeaderFullEdit {
    pub id: i32,
    pub user: Option<Vec<Uuid>>,
    pub team: Option<i32>,
    pub is_ab_team: Option<bool>,
    pub match_id: Option<i32>,
    pub set: Option<i32>,
    pub event_code: Option<String>,
    pub tournament_level: Option<TournamentLevels>,
    pub station: Option<Stations>,
    pub created_at: Option<DateTime<Local>>,
    pub is_mvp: Option<bool>,
    pub defence: Option<f32>,
    pub comment: Option<String>
}

async fn to_full_am(header: HeaderFullEdit, db: &DatabaseConnection) -> Result<genertic_header::ActiveModel, DbErr> {
    //get gametype and game id (for later insert into game)
    let game_model = match genertic_header::Entity::find_by_id(header.id).one(db).await? {
        Some(a) => a,
        None => {
            return Err(DbErr::Custom("Not a vaild ID".to_string()));
        },
    };
    
    Ok(genertic_header::ActiveModel {
        id: NotSet,
        team: header.team.map(Set).unwrap_or(NotSet),
        is_ab_team: header.is_ab_team.map(Set).unwrap_or(NotSet),
        match_id: header.match_id.map(Set).unwrap_or(NotSet),
        set: header.set.map(Set).unwrap_or(NotSet),
        total_score: NotSet, //Will be set later
        event_code: header.event_code.map(Set).unwrap_or(NotSet),
        tournament_level: header.tournament_level.map(Set).unwrap_or(NotSet),
        station: header.station.map(Set).unwrap_or(NotSet),
        created_at: header.created_at.map(Set).unwrap_or(NotSet),
        is_mvp: header.is_mvp.map(Set).unwrap_or(NotSet),
        game_type_id: Set(game_model.game_type_id),
        game_id: Set(game_model.game_id),
        teleop_score: NotSet,
        auto_score: NotSet,
        defence: header.defence.map(Set).unwrap_or(NotSet),
        comment: header.comment.map(Set).unwrap_or(NotSet),
        user: header.user.map(Set).unwrap_or(NotSet),
    })
}

//REAL functions that help grab the value

pub async fn insert_game(data: &GamesInserts, db: &DatabaseConnection) -> Result<i32, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_insert_game(data, game, db).await
}

pub async fn graph_game(team: &i32, is_ab_team: &bool, event_code: &Option<String>, db: &DatabaseConnection) -> Result<Vec<GamesGraph>, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    let mut command = genertic_header::Entity::find()
        .filter(genertic_header::Column::Team.eq(*team))
        .filter(genertic_header::Column::IsAbTeam.eq(*is_ab_team))
        .filter(genertic_header::Column::GameTypeId.eq(game.get_year_id()));
    if let Some(e) = event_code {
        command = command.filter(genertic_header::Column::EventCode.eq(e));
    }
    let res: Vec<GamesGraph> = command
        .select_only()
        .column_as(genertic_header::Column::CreatedAt, "time")
        .column_as(genertic_header::Column::TotalScore, "total_score")
        .column_as(genertic_header::Column::AutoScore, "auto_score")
        .column_as(genertic_header::Column::TeleopScore, "teleop_score")
        .column_as(genertic_header::Column::Defence, "defence")
        .into_model::<GamesGraph>()
        .all(db)
    .await?;

    Ok(res)
}

pub async fn search_game(param: &SearchParam, db: &DatabaseConnection) -> Result<Vec<GamesFull>, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_search_game(game, param, db).await
}

pub async fn average_game(event_code: &String, db: &DatabaseConnection) -> Result<Vec<TeamAvg>, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_average_game(game, event_code, db).await
}

pub async fn frontrunner(games: &FrontRunnerGame) -> Result<FrontRunnerReturn, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    game.frontrunner_op(games)
}

#[allow(dead_code)]
pub async fn get_game(id: i32, db: &DatabaseConnection) -> Result<GamesFull, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_get_game(game, id, db).await
}

pub async fn delete_game(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    let game = game_dispatch(SETTINGS.year);

    game.delete(id, db).await
}


define_games!(
    //Insert each year here
    RebuiltGame => crate::backenddb::entrys::rebuilt
);  
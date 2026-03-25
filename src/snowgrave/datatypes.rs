//Some termioinlgys
//Match = each data entry for each team
//game = whole thing for a game

use std::collections::HashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, prelude::DateTimeLocal};

use crate::{
    backenddb::game::GamesFullSpecific, entity::{
        game_scouts, mvp_data, mvp_scouters, sea_orm_active_enums::{Stations, TournamentLevels}, upcoming_game, upcoming_team
    }
};

//basic types
#[derive(Hash, PartialEq, Eq, Clone, Copy, JsonSchema, Serialize, Deserialize, Debug)]
pub struct Team {
    pub number: i32,
    pub is_ab_team: bool,
}

//Primtive Mvp types
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MvpUpcoming {
    pub id: i32, //upcoming id
    pub name: Uuid,
    pub data: Option<MvpData>
}

pub struct MvpUpcomingFull {
    pub id: i32, //upcoming id
    pub name: Uuid,
    pub data: MvpData
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct MvpData {
    pub mvp_team: Team,
    pub comment: String,
    pub is_blue: bool,
    pub total_score: i32,
    pub penalty_score: i32, 
}

//The base Pair type

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MvpPair {
    pub red: MvpUpcoming,
    pub blue:  MvpUpcoming
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MvpPairPart {
    pub red: Option<MvpUpcoming>,
    pub blue:  Option<MvpUpcoming>
}

pub struct MvpPairFull {
    pub red: MvpUpcomingFull,
    pub blue:  MvpUpcomingFull
}

//Scout type

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ScoutMatch {
    pub id: i32, //of game scout's
    pub name: Uuid,
    pub station: Stations,
    pub done: bool,
    pub is_redo: bool,
    pub team: Team,
    pub data: Option<ScoutMatchData>
}


#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct FailerInfo {
    pub name: Uuid,
    pub station: Stations,
    pub team: Team,
    pub upcoming_scout_id: i32,
}

pub struct ScoutMatchFull {
    pub id: i32,
    pub name: Uuid,
    pub station: Stations,
    pub done: bool,
    pub is_redo: bool,
    pub team: Team,
    pub data: ScoutMatchData
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct ScoutMatchData {
    pub match_id: i32,
    pub set: i32,
    pub total_score: i32,
    pub teleop_score: i32,
    pub auto_score: i32,
    pub defence: i32,
    pub comment: String,
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    pub created_at: DateTimeLocal,
    pub game: GamesFullSpecific,
    pub game_id: i32,
    pub game_type_id: i32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ScoutGame { 
    pub red_1: Vec<ScoutMatch>,
    pub red_2: Vec<ScoutMatch>,
    pub red_3: Vec<ScoutMatch>,
    pub blue_1: Vec<ScoutMatch>,
    pub blue_2: Vec<ScoutMatch>,
    pub blue_3: Vec<ScoutMatch>,
}

pub struct ScoutGameFull { 
    pub red_1: Vec<ScoutMatchFull>,
    pub red_2: Vec<ScoutMatchFull>,
    pub red_3: Vec<ScoutMatchFull>,
    pub blue_1: Vec<ScoutMatchFull>,
    pub blue_2: Vec<ScoutMatchFull>,
    pub blue_3: Vec<ScoutMatchFull>,
}

//Game

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Game {
    pub event_code: String,
    pub match_id: i32,
    pub set: i32,
    pub tournament_level: TournamentLevels,
    pub scout: ScoutGame,
    pub mvp: MvpPair,
}


#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GamePart {
    pub id: i32,
    pub event_code: String,
    pub match_id: i32,
    pub set: i32,
    pub tournament_level: TournamentLevels,
    pub scout: ScoutGame,
    pub mvp: MvpPairPart,
}

pub struct GameFull {
    pub event_code: String,
    pub match_id: i32,
    pub set: i32,
    pub tournament_level: TournamentLevels,
    pub scout: ScoutGameFull,
    pub mvp: MvpPairFull,
}
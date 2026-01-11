use std::num::ParseIntError;
use std::ops::Not;

use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::{Database, DatabaseConnection, DbErr, EntityTrait};

use crate::backenddb::game;
use crate::entity::sea_orm_active_enums::{Stations, TournamentLevels};
use crate::entity::upcoming_game::ActiveModel;
use crate::pit::create_pit_upcoming;
use crate::snowgrave::blue::{self, TbaMatch, pull_from_blue};
use crate::entity::{upcoming_game, upcoming_team};

pub enum Blue2DBErr {
    FailedToFindRightTourLevel(String),
    FailedToParseTeam(ParseIntError),
    InvaildStation(usize),
    DbErr(DbErr)
}

pub fn blue_to_db_tour_level(data: &str) -> Result<TournamentLevels, Blue2DBErr> {
    match data {
        "qm" => Ok(TournamentLevels::QualificationMatch),
        "sf" => Ok(TournamentLevels::Semifinal),
        "f" => Ok(TournamentLevels::Final),
        "qf" => Ok(TournamentLevels::Quarterfinal), 
        a => {
            Err(Blue2DBErr::FailedToFindRightTourLevel(a.to_string()))
        }
    }
}

pub fn into_snow(data: &TbaMatch, event_code: &String) -> Result<(upcoming_game::ActiveModel), Blue2DBErr> {
    let level = blue_to_db_tour_level(&data.comp_level)?;

    Ok(ActiveModel {
        id: NotSet, //By db
        event_code: Set(event_code.clone()),
        match_id: Set(data.match_number),
        set: Set(data.set_number),
        tournament_level: Set(level),
        mvp_id_blue: Set(None),
        mvp_id_red: Set(None),
        
    })
}

pub fn into_snow_team(data: &TbaMatch, id: i32) -> Result<Vec<upcoming_team::ActiveModel>, Blue2DBErr> {
    let mut teams: Vec<upcoming_team::ActiveModel> = Vec::with_capacity(data.alliances.red.team_keys.len()+data.alliances.blue.team_keys.len());
    for (i, name) in data.alliances.red.team_keys.iter().enumerate() {

        let is_b_team;
        let team_string: String;
        if name.ends_with('B') {
            is_b_team = true;
            team_string = name[3..name.len() - 1].to_string();
        } else {
            is_b_team = false;
            team_string = name[3..].to_string();
        };
        let team_number: i32 = match team_string.parse() {
            Ok(a) => a,
            Err(a) => {
                return Err(Blue2DBErr::FailedToParseTeam(a));
            }
        };
        let station = match i {
            1 => Stations::Red1,
            2 => Stations::Red2,
            3 => Stations::Red3,
            _ => {
                return Err(Blue2DBErr::InvaildStation(i));
            }
        };
        let team_data: upcoming_team::ActiveModel = upcoming_team::ActiveModel {
            id: NotSet,
            station: Set(station),
            is_ab_team: Set(is_b_team),
            team: Set(team_number),
            game_id: Set(id),
        };
        teams.push(team_data);

    }
    
    //Blue
    for (i, name) in data.alliances.blue.team_keys.iter().enumerate() {

        let is_b_team;
        let team_string: String;
        if name.ends_with('B') {
            is_b_team = true;
            team_string = name[3..name.len() - 1].to_string();
        } else {
            is_b_team = false;
            team_string = name[3..].to_string();
        };
        let team_number: i32 = match team_string.parse() {
            Ok(a) => a,
            Err(a) => {
                return Err(Blue2DBErr::FailedToParseTeam(a));
            }
        };
        let station = match i {
            1 => Stations::Blue1,
            2 => Stations::Blue2,
            3 => Stations::Blue3,
            _ => {
                return Err(Blue2DBErr::InvaildStation(i));
            }
        };
        let team_data: upcoming_team::ActiveModel = upcoming_team::ActiveModel {
            id: NotSet,
            station: Set(station),
            is_ab_team: Set(is_b_team),
            team: Set(team_number),
            game_id: Set(id),
        };
        teams.push(team_data);

    }
   
    Ok(teams)
}

pub async fn queue_snow(games: Vec<blue::TbaMatch>, event_code: &String, client: &reqwest::Client, db: &DatabaseConnection) -> Result<(), Blue2DBErr> {
    let mut games_ids: Vec<i32> = Vec::new();
    for game in &games {
        let res = into_snow(&game, event_code)?;
        let db_res = match upcoming_game::Entity::insert(res).exec(db).await {
            Ok(a) => a,
            Err(a) => {
                return Err(Blue2DBErr::DbErr(a));
            }
        }.last_insert_id;
        games_ids.push(db_res);
    }
    for team in games.iter().zip(games_ids.iter()) {
        let res = into_snow_team(team.0, *team.1)?;
        match upcoming_team::Entity::insert_many(res).exec(db).await {
            Ok(_) => {},
            Err(a) => {
                return Err(Blue2DBErr::DbErr(a));
            },
        };
    }
    //for pit
    create_pit_upcoming::create_pit_upcoming(db,event_code).await.map_err(|x| Blue2DBErr::DbErr(x))?;


    Ok(())
}
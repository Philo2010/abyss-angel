use std::num::ParseIntError;
use std::ops::Not;

use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::{Database, DatabaseConnection, DbErr, EntityTrait};

use crate::backenddb::game;
use crate::entity::sea_orm_active_enums::{Stations, TournamentLevels};
use crate::entity::upcoming_game::ActiveModel;
use crate::snowgrave::blue::{self, TbaMatch, pull_from_blue};
use crate::entity::{upcoming_game, upcoming_team};
use crate::snowgrave::snowgrave_que::{into_snow, into_snow_team};
use crate::snowgrave::snowgrave_que::Blue2DBErr;


pub async fn queue_snow(games: Vec<blue::TbaMatch>, event_code: &String, db: &DatabaseConnection) -> Result<(), Blue2DBErr> {
    let mut games_ids: Vec<i32> = Vec::new();
    for game in &games {
        let res = into_snow(&game, event_code)?;
        if res.tournament_level.clone().unwrap() == TournamentLevels::QualificationMatch {
            continue;
        }
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


    Ok(())
}
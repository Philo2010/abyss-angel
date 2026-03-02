use std::collections::HashSet;

use sea_orm::{DatabaseConnection, DbErr};
use uuid::Uuid;

use crate::{SETTINGS, backenddb::game::{GamesFullSpecific, frontrunner, game_dispatch, get_game}, snowgrave::{self, check_if_filled::check_if_filled, datatypes::{GameFull, ScoutMatchFull}}};
const RATIO_FALLED: f32 = 0.8;

pub struct FinalCheck {
    blue1: ScoutMatchFull,
    blue2: ScoutMatchFull,
    blue3: ScoutMatchFull,
    red1: ScoutMatchFull,
    red2: ScoutMatchFull,
    red3: ScoutMatchFull,
}

pub enum CheckRet {
    Failed(Vec<usize>),
    Passed(GamesFullSpecific)
}

async fn check_game(game_data: Vec<&GamesFullSpecific>) -> Result<CheckRet, DbErr> {
    let res = frontrunner(&game_data).await?;
    let ratio =  res.crazy.len() as f32/game_data.len() as f32;
    let crazy_set: HashSet<usize> = res.crazy.iter().copied().collect();
    if ratio >= RATIO_FALLED {
        //passed
        return Ok(CheckRet::Passed(res.avg));
    } else {
        //failed
        return Ok(CheckRet::Failed(res.crazy));
    }
}

pub async fn precheck(upcoming_game_id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {

    //get data from db
    let game_not_cool = snowgrave::db_models_to_snow::get_game(upcoming_game_id, db).await?;
    let game = match check_if_filled(game_not_cool) {
        super::check_if_filled::FilledCheck::Filled(game_full) => {
            game_full
        },
        super::check_if_filled::FilledCheck::NotFilled => {
            return Err(DbErr::Custom("Not done!".to_string()));
        },
    };

    //run frontrunner
    
    let blue1s_game: Vec<&GamesFullSpecific> = game.scout.blue_1.iter().map(|x| &x.data.game).collect();
    


    Ok(())
}
use sea_orm::{DatabaseConnection, DbErr};
use uuid::Uuid;

use crate::{backenddb::game::GamesInserts, snowgrave::{self, check_system::{check_mid::check_mid, check_if_filled::check_if_filled, precheck::precheck}, datatypes::FailerInfo}};




pub enum CheckBindReturn {
    Passed(Vec<GamesInserts>, Vec<FailerInfo>),
    Failed(Vec<FailerInfo>),
    NotDone,
}

pub async fn check_bind(upcoming_game_id: i32, db: &DatabaseConnection) -> Result<CheckBindReturn, DbErr> {
    let game_not_cool = snowgrave::db_models_to_snow::get_game(upcoming_game_id, db).await?;
    let game = match check_if_filled(game_not_cool) {
        super::check_if_filled::FilledCheck::Filled(game_full) => game_full,
        super::check_if_filled::FilledCheck::NotFilled => {
            return Ok(CheckBindReturn::NotDone);
        },
    };

    let data = match precheck(&game).await? {
        super::precheck::PreCheckReturn::Passed(pre_check_game, mut error) => {
            //run the new check
            let res = check_mid(&pre_check_game);
            match res {
                super::check_mid::CheckReturn::Red => {
                    let mut red_uuids: Vec<FailerInfo> = Vec::new();
                    red_uuids.append(&mut game.scout.red_1.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    red_uuids.append(&mut game.scout.red_2.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    red_uuids.append(&mut game.scout.red_3.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    //TODO: Return
                    for uuid in red_uuids {
                        error.insert(uuid);
                    }
                    let mut games = vec![pre_check_game.blue1, pre_check_game.blue2, pre_check_game.blue3];

                    return Ok(CheckBindReturn::Passed(games, error.into_iter().collect()));
                },
                super::check_mid::CheckReturn::RedAndBlue => {
                    let mut all_uuids: Vec<FailerInfo> = Vec::new();
                    all_uuids.append(&mut game.scout.red_1.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    all_uuids.append(&mut game.scout.red_2.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    all_uuids.append(&mut game.scout.red_3.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    all_uuids.append(&mut game.scout.blue_1.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    all_uuids.append(&mut game.scout.blue_2.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    all_uuids.append(&mut game.scout.blue_3.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    for uuid in all_uuids {
                        error.insert(uuid);
                    }

                    return Ok(CheckBindReturn::Failed(error.into_iter().collect()));
                },
                super::check_mid::CheckReturn::Blue => {
                    let mut blue_uuids: Vec<FailerInfo> = Vec::new();
                    blue_uuids.append(&mut game.scout.blue_1.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    blue_uuids.append(&mut game.scout.blue_2.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    blue_uuids.append(&mut game.scout.blue_3.iter().map(|x| FailerInfo {
                        name: x.name,
                        station: x.station,
                        team: x.team,
                        upcoming_scout_id: x.id
                    }).collect());
                    for uuid in blue_uuids {
                        error.insert(uuid);
                    }

                    let mut games = vec![pre_check_game.red1, pre_check_game.red2, pre_check_game.red3];

                    return Ok(CheckBindReturn::Passed(games, error.into_iter().collect()));
                    
                },
                super::check_mid::CheckReturn::NoFail => {
                    let mut games = vec![pre_check_game.red1, pre_check_game.red2, pre_check_game.red3, pre_check_game.blue1, pre_check_game.blue2, pre_check_game.blue3];

                    return Ok(CheckBindReturn::Passed(games, error.into_iter().collect()));
                },
            };
        },
        super::precheck::PreCheckReturn::Failed(error) => {
            return Ok(CheckBindReturn::Failed(error.into_iter().collect()));
        },
        super::precheck::PreCheckReturn::NotDone => {
            return Ok(CheckBindReturn::NotDone); //No need to error
        },
    };
}
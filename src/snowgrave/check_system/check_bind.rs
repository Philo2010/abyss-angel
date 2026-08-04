use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ExprTrait, QueryFilter, QuerySelect};
use sea_orm::sea_query::Alias;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{backenddb::game::{GamesInserts, game_dispatch}, entity::{genertic_header, sea_orm_active_enums::Stations}, SETTINGS, snowgrave::{self, check_system::{check_mid::check_mid, check_if_filled::check_if_filled, precheck::{precheck, PreCheckGame}}, datatypes::FailerInfo}};

fn same_alliance(a: Stations, b: Stations) -> bool {
    matches!(a, Stations::Red1 | Stations::Red2 | Stations::Red3) == matches!(b, Stations::Red1 | Stations::Red2 | Stations::Red3)
}

async fn team_avg_total_scores(event_code: &str, year_id: i32, db: &DatabaseConnection) -> Result<HashMap<(i32, bool), f32>, DbErr> {
    let rows: Vec<(f64, i32, bool)> = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(year_id))
        .filter(genertic_header::Column::EventCode.eq(event_code))
        .select_only()
        .column_as(genertic_header::Column::TotalScore.avg().cast_as(Alias::new("FLOAT8")), "total_score")
        .column(genertic_header::Column::Team)
        .column(genertic_header::Column::IsAbTeam)
        .group_by(genertic_header::Column::Team)
        .group_by(genertic_header::Column::IsAbTeam)
        .into_tuple()
        .all(db).await?;
    Ok(rows.into_iter().map(|(score, team, is_ab_team)| ((team, is_ab_team), score as f32)).collect())
}

/// DPDG is only created for the main defender and equals the sum of the opposing
/// teams' total scores for this match minus their event averages. Computed here,
/// during the checking phase, since that's the only time all opponent data is present.
async fn stamp_dpdg(all_six: &mut PreCheckGame, db: &DatabaseConnection) -> Result<(), DbErr> {
    let model = game_dispatch(SETTINGS.year);
    let team_avg = team_avg_total_scores(&all_six.red1.header.event_code, model.get_year_id(), db).await?;

    let all: [&GamesInserts; 6] = [
        &all_six.red1, &all_six.red2, &all_six.red3,
        &all_six.blue1, &all_six.blue2, &all_six.blue3,
    ];
    let all_scores: Vec<f32> = all.iter().map(|g| model.get_scores(&g.game).total_score).collect();
    let all_info: Vec<(Stations, i32, bool)> = all.iter()
        .map(|g| (g.header.station, g.header.team, g.header.is_ab_team))
        .collect();

    let dpdgs: Vec<Option<f32>> = all.iter().enumerate().map(|(i, g)| {
        if !model.main_defender(&g.game) {
            return None;
        }
        let value: f32 = all_info.iter().enumerate()
            .filter(|(_, (station, _, _))| !same_alliance(*station, all_info[i].0))
            .map(|(j, (_, team, is_ab_team))| {
                let avg = team_avg.get(&(*team, *is_ab_team)).copied().unwrap_or(0.0);
                if avg == 0.0 {
                    0.0
                } else {
                    (avg - all_scores[j]) / avg * 100.0
                }
            })
            .sum();
        Some(value)
    }).collect();

    for (game, dpdg) in [
        &mut all_six.red1, &mut all_six.red2, &mut all_six.red3,
        &mut all_six.blue1, &mut all_six.blue2, &mut all_six.blue3,
    ].into_iter().zip(dpdgs) {
        model.set_dpdg(&mut game.game, dpdg);
    }

    Ok(())
}




#[derive(Debug)]
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
        super::precheck::PreCheckReturn::Passed(mut pre_check_game, mut error) => {
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
                    stamp_dpdg(&mut pre_check_game, db).await?;

                    let games = vec![pre_check_game.blue1, pre_check_game.blue2, pre_check_game.blue3];

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

                    stamp_dpdg(&mut pre_check_game, db).await?;

                    let games = vec![pre_check_game.red1, pre_check_game.red2, pre_check_game.red3];

                    return Ok(CheckBindReturn::Passed(games, error.into_iter().collect()));
                    
                },
                super::check_mid::CheckReturn::NoFail => {
                    stamp_dpdg(&mut pre_check_game, db).await?;

                    let games = vec![pre_check_game.red1, pre_check_game.red2, pre_check_game.red3, pre_check_game.blue1, pre_check_game.blue2, pre_check_game.blue3];

                    return Ok(CheckBindReturn::Passed(games, error.into_iter().collect()));
                },
            };
        },
        super::precheck::PreCheckReturn::Failed(error) => {
            return Ok(CheckBindReturn::Failed(error.into_iter().collect()));
        },
    };
}
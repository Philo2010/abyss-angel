use std::collections::HashSet;

use sea_orm::{DatabaseConnection, DbErr};
use uuid::Uuid;

use crate::{SETTINGS, backenddb::game::{FrontRunnerGame, FrontRunnerReturn, GamesFull, GamesFullSpecific, GamesInserts, GamesInsertsSpecific, HeaderInsert, frontrunner, game_dispatch, get_game}, entity::sea_orm_active_enums::Stations, snowgrave::{self, check_system::check_if_filled::check_if_filled, datatypes::{FailerInfo, GameFull, MvpPairFull, ScoutMatchFull}}};

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
    Passed(GamesInserts)
}

async fn check_game(game_data: FrontRunnerGame) -> Result<CheckRet, DbErr> {
    let res = frontrunner(&game_data).await?;
    let ratio =  res.crazy.len() as f32/game_data.games.len() as f32;
    let crazy_set: HashSet<usize> = res.crazy.iter().copied().collect();
    eprintln!(
        "[precheck] team={} station={:?} scouts={} outliers={} ratio={:.2} threshold={:.2} => {}",
        game_data.team,
        game_data.station,
        game_data.games.len(),
        res.crazy.len(),
        ratio,
        super::OUTLIER_FAIL_RATIO,
        if ratio >= super::OUTLIER_FAIL_RATIO { "FAIL" } else { "PASS" }
    );
    if ratio >= super::OUTLIER_FAIL_RATIO {
        //too many outliers, not enough good data
        return Ok(CheckRet::Failed(res.crazy));
    } else {
        //enough good data, use the average
        if let Some(res) = res.avg {
            return Ok(CheckRet::Passed(res));
        } else {
            //data was not good engpoih to form 
            return Ok(CheckRet::Failed(res.crazy));
        }
    }
}

enum ProcessPos {
    Passed(GamesInserts),
    Failed //Added to the errors hash
}

async fn process_position(
    scouts: &[ScoutMatchFull],
    mvp_pair: &MvpPairFull,
    errors: &mut HashSet<FailerInfo>,
) -> Result<Option<GamesInserts>, DbErr> {
    //find mvp status
    let is_mvp: bool;
    if scouts.first().unwrap().station == Stations::Red1 ||
       scouts.first().unwrap().station == Stations::Red2 ||
       scouts.first().unwrap().station == Stations::Red3 {
        if scouts.first().unwrap().team == mvp_pair.red.data.mvp_team {
            is_mvp = true;
        } else {
            is_mvp = false
        }
    } else {
        if scouts.first().unwrap().team == mvp_pair.blue.data.mvp_team {
            is_mvp = true;
        } else {
            is_mvp = false
        }
    }


    let games: FrontRunnerGame = FrontRunnerGame {
        games: scouts.iter().map(|x| (x.name, x.data.game.clone())).collect(),
        defence: scouts.iter().map(|x| x.data.defence as f32).collect(),
        comment: scouts.iter().map(|x| x.data.comment.clone()).collect(),
        team: scouts.first().unwrap().team.number,
        is_ab_team: scouts.first().unwrap().team.is_ab_team,
        match_id: scouts.first().unwrap().data.match_id,
        set: scouts.first().unwrap().data.set,
        event_code: scouts.first().unwrap().data.event_code.clone(),
        tournament_level: scouts.first().unwrap().data.tournament_level,
        station: scouts.first().unwrap().station,
        is_mvp: is_mvp
    };
    match check_game(games).await? {
        CheckRet::Passed(result) => Ok(Some(result)),
        CheckRet::Failed(items) => {
            for item in items {
                let scout = scouts.get(item)
                    .unwrap_or_else(|| panic!("Scout entry missing at index {item}"));
                errors.insert(FailerInfo {
                    name: scout.name,
                    station: scout.station,
                    team: scout.team,
                    upcoming_scout_id: scout.id,
                });
            }
            Ok(None)
        }
    }
}


pub struct PreCheckGame {
    pub red1: GamesInserts,
    pub red2: GamesInserts,
    pub red3: GamesInserts,
    pub blue1: GamesInserts,
    pub blue2: GamesInserts,
    pub blue3: GamesInserts,
    pub penalty_score_red: i32,
    pub total_score_red: i32,
    pub penalty_score_blue: i32,
    pub total_score_blue: i32,
}

pub enum PreCheckReturn {
    Passed(PreCheckGame, HashSet<FailerInfo>),
    Failed(HashSet<FailerInfo>),
}


pub async fn precheck(game: &GameFull) -> Result<PreCheckReturn, DbErr> {

    let mut errors: HashSet<FailerInfo> = HashSet::new();

    let blue1 = process_position(&game.scout.blue_1, &game.mvp, &mut errors).await?;
    let blue2 = process_position(&game.scout.blue_2, &game.mvp, &mut errors).await?;
    let blue3 = process_position(&game.scout.blue_3, &game.mvp, &mut errors).await?;
    let red1 = process_position(&game.scout.red_1, &game.mvp, &mut errors).await?;
    let red2 = process_position(&game.scout.red_2, &game.mvp, &mut errors).await?;
    let red3 = process_position(&game.scout.red_3, &game.mvp, &mut errors).await?;


    if !errors.is_empty() {
        return Ok(PreCheckReturn::Failed(errors));
    }

    Ok(PreCheckReturn::Passed({
        PreCheckGame { 
            red1: red1.unwrap(),
            red2: red2.unwrap(),
            red3: red3.unwrap(),
            blue1: blue1.unwrap(),
            blue2: blue2.unwrap(),
            blue3: blue3.unwrap(),
            total_score_red: game.mvp.red.data.total_score,
            penalty_score_red: game.mvp.red.data.penalty_score,
            total_score_blue: game.mvp.blue.data.total_score,
            penalty_score_blue: game.mvp.blue.data.penalty_score,
        }
    },
    errors
    )) // safe — errors empty means all Ok
}
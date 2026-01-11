use std::collections::HashMap;

use crate::entity::sea_orm_active_enums::Stations;
use crate::snowgrave::{check_complete::CheckMatchErr, datatypes::GameFull};
use crate::snowgrave::datatypes::{ScouterWithScore, MvpScouter};


const AGREE_AMOUNT: f32 = 0.8;
//just something i found off the net (lol)
fn most_common<T: Eq + std::hash::Hash + Copy>(items: &[T]) -> Option<(T, usize)> {
    let mut counts = HashMap::new();

    for &item in items {
        *counts.entry(item).or_insert(0) += 1;
    }

    counts.into_iter().max_by_key(|&(_, count)| count)
}

pub struct CheckFailerReturn {
    pub game_number: i32,
    pub teams_to_redo: Vec<i32>,
    pub reasons: Vec<ScouterWithScore>,
    pub winner_teams: Vec<i32>
}


pub fn check(game: &GameFull) -> Result<CheckFailerReturn, CheckMatchErr> {

    let mut fails: Vec<ScouterWithScore> = Vec::new();
    let mut teams_to_redo: Vec<i32> = Vec::new();
    let mut red_score: i32 = 0;
    let mut red_teams: Vec<i32> = Vec::new();
    let mut blue_score: i32 = 0;
    let mut blue_teams: Vec<i32> = Vec::new();
    let mut extra_check = true;
    let mut scouts_id_to_non_dup: Vec<i32> = Vec::new(); //is the SNOWGRAVE id, must be converted into and duped by cast

    for team in &game.teams.0 {

        let scores: Vec<i32> = team.scouters.iter().map(|x| x.total_score).collect();
        let cooldata: (i32, usize) = most_common(&scores).ok_or(CheckMatchErr::NotAllScoutersAreDone)?;


        let winner_id = team
            .scouters
            .iter()
            .find(|s| s.total_score == cooldata.0)
            .unwrap()
            .id;

        for idx in &team.scouters {
            if idx.total_score != cooldata.0 {
                fails.push(*idx);
            }
        }


        let ratio: f32 = cooldata.1 as f32 / scores.len() as f32;
        if ratio < AGREE_AMOUNT {
            teams_to_redo.push(team.id);
            //entire team gets slimed
            fails.clear();
            for idx in &team.scouters {
                fails.push(*idx);
            }
            extra_check = false;
        } else {
            //we dont care what person will be credited so ya...
            scouts_id_to_non_dup.push(winner_id);
            //only scouter that fails gets slimed
            for idx in &team.scouters {
                if idx.total_score != cooldata.0 {
                    fails.push(*idx);
                }
            }
            //its ok
            if team.station == Stations::Red1 || team.station == Stations::Red2 || team.station == Stations::Red3 {
                red_score = cooldata.0 + red_score;
                red_teams.push(team.id);
            } else {
                blue_score = cooldata.0 + blue_score;
                blue_teams.push(team.id);
            }
        }
    }

    //check MVP along with scores
    let red_mvp_score = game.mvp.red.data.total_score - game.mvp.red.data.penalty_score;
    let blue_mvp_score = game.mvp.blue.data.total_score- game.mvp.blue.data.penalty_score;

    if !extra_check {
        //we are done there is nothing else we can check
        let res = CheckFailerReturn {
            game_number: game.match_id,
            teams_to_redo,
            reasons: fails,
            winner_teams: scouts_id_to_non_dup
        };
        return Ok(res);
    }

    if red_score != red_mvp_score {
        //red allence has failed
        teams_to_redo.append(&mut red_teams);
        for team in &game.teams.0 {
            for scouter in &team.scouters {
                if (scouter.station == Stations::Red1 || scouter.station == Stations::Red2 || scouter.station == Stations::Red3) {
                    if !fails.contains(scouter) {
                        fails.push(*scouter);
                    }
                }
            }
        }
    }
    if blue_score != blue_mvp_score {
        teams_to_redo.append(&mut blue_teams);
        for team in &game.teams.0 {
            for scouter in &team.scouters {
                if (scouter.station == Stations::Blue1 || scouter.station == Stations::Blue2 || scouter.station == Stations::Blue3) {
                    if !fails.contains(scouter) {
                        fails.push(*scouter);
                    }
                }
            }
        }
    }

    let res = CheckFailerReturn {
            game_number: game.match_id,
            teams_to_redo,
            reasons: fails,
            winner_teams: scouts_id_to_non_dup
    };

    Ok(res)
}
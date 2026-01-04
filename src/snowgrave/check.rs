use std::collections::HashMap;

use crate::entity::sea_orm_active_enums::Stations;
use crate::snowgrave::{check_complete::CheckMatchErr, datatypes::GameFull};
use crate::snowgrave::datatypes::{ScouterWithScore, MvpScouter};


const AGREE_AMOUNT: f32 = 0.8;
pub enum CheckFailerReason {
    NoOne,
    Mvp(MvpScouter),
    Scouter(ScouterWithScore)
}


//just something i found off the net (lol)
fn most_common<T: Eq + std::hash::Hash + Copy>(items: &[T]) -> Option<(T, usize)> {
    let mut counts = HashMap::new();

    for &item in items {
        *counts.entry(item).or_insert(0) += 1;
    }

    counts.into_iter().max_by_key(|&(_, count)| count)
}

pub struct CheckFailerReturn {
    pub teams_to_redo: Vec<i32>,
    pub redo_mvp: bool,
    pub reasons: Vec<CheckFailerReason>
}


pub fn check(game: &GameFull) -> Result<CheckFailerReturn, CheckMatchErr> {

    let mut fails: Vec<CheckFailerReason> = Vec::new();
    let mut teams_to_redo: Vec<i32> = Vec::new();
    let mut red_score: i32 = 0;
    let mut red_teams: Vec<i32> = Vec::new();
    let mut blue_score: i32 = 0;
    let mut blue_teams: Vec<i32> = Vec::new();
    let mut extra_check = true;
    let mut redo_mvp = false;

    for team in &game.teams.0 {

        let scores: Vec<i32> = team.scouters.iter().map(|x| x.total_score).collect();

        let cooldata: (i32, usize) = most_common(&scores).ok_or(CheckMatchErr::NotAllScoutersAreDone)?;



        for idx in &team.scouters {
            if idx.total_score != cooldata.0 {
                fails.push(CheckFailerReason::Scouter(*idx));
            }
        }


        let ratio: f32 = cooldata.1 as f32 / scores.len() as f32;
        if ratio < AGREE_AMOUNT {
            teams_to_redo.push(team.id);
            extra_check = false;
        } else {
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
    let red_mvp_score = game.mvp.data.total_score_for_red - game.mvp.data.penalty_score_for_red;
    let blue_mvp_score = game.mvp.data.total_score_for_blue - game.mvp.data.penalty_score_for_blue;

    if !extra_check {
        //we are done there is nothing else we can check
        let res = CheckFailerReturn {
            teams_to_redo,
            redo_mvp: false, //we just assume its ok for now
            reasons: fails,
        };
        return Ok(res);
    }

    if red_score != red_mvp_score {
        //red allence has failed
        teams_to_redo.append(&mut red_teams);
        redo_mvp = true;
    }
    if blue_score != blue_mvp_score {
        teams_to_redo.append(&mut blue_teams);
        redo_mvp = true;
    }

    let res = CheckFailerReturn {
            teams_to_redo,
            redo_mvp: redo_mvp, //we just assume its ok for now
            reasons: fails,
    };

    Ok(res)
}
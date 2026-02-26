use std::collections::HashMap;

use crate::entity::sea_orm_active_enums::Stations;
use crate::snowgrave::find_point_distance::{CheckReturn, check_pass};
use crate::snowgrave::{check_complete::CheckMatchErr, datatypes::GameFull};
use crate::snowgrave::datatypes::ScouterWithScore;


const AGREE_AMOUNT: f32 = 0.8;
//just something i found off the net (lol)
fn most_common<T: Eq + std::hash::Hash + Copy>(items: &[T]) -> Option<(T, usize)> {
    let mut counts = HashMap::new();

    for &item in items {
        *counts.entry(item).or_insert(0) += 1;
    }

    counts.into_iter().max_by_key(|&(_, count)| count)
}

#[derive(Debug)]
pub struct CheckFailerReturn {
    pub game_number: i32,
    pub teams_to_redo: Vec<i32>,
    pub scouts_to_forgive: Vec<ScouterWithScore>,
    pub reasons: Vec<ScouterWithScore>,
    pub winner_teams: Vec<i32>
}


pub fn check(game: &GameFull) -> Result<CheckFailerReturn, CheckMatchErr> {
    use std::collections::HashSet;

    let mut fails = Vec::new();
    let mut forgive = Vec::new();
    let mut teams_to_redo = Vec::new();
    let mut winner_teams = Vec::new();
    let mut failed_ids = HashSet::new();

    let mut red_sum = 0;
    let mut blue_sum = 0;
    let mut trusted_red = Vec::new();
    let mut trusted_blue = Vec::new();

    // ---------- TEAM CONSENSUS ----------
    for team in &game.teams.0 {
        let scores: Vec<i32> = team.scouters.iter().map(|s| s.total_score).collect();

        let result = check_pass(scores);

        let passed = result.passed;
        let failed = result.failed;

        let ratio = passed.len() as f32 / team.scouters.len() as f32;

        if ratio < AGREE_AMOUNT {
            teams_to_redo.push(team.id);
            for s in team.scouters.iter().copied() {
                if failed_ids.insert(s.id) {
                    fails.push(s);
                }
            }
            continue;
        }

        // Forgive passed
        for &i in &passed {
            let s = team.scouters[i];
            forgive.push(s);
        }

        // Fail failed
        for &i in &failed {
            let s = team.scouters[i];
            if failed_ids.insert(s.id) {
                fails.push(s);
            }
        }

        // Use average (or first) passing score for alliance sum
        let team_score: i32 =
            passed.iter().map(|&i| team.scouters[i].total_score).sum();

        match team.station {
            Stations::Red1 | Stations::Red2 | Stations::Red3 => {
                red_sum += team_score;
                trusted_red.push(team.id);
            }
            _ => {
                blue_sum += team_score;
                trusted_blue.push(team.id);
            }
        }

        // pick lowest ID among passed
        if let Some(min_id) =
            passed.iter().map(|&i| team.scouters[i].id).min()
        {
            winner_teams.push(min_id);
        }
    }

    // ---------- MVP CHECK (CERTAINTY ONLY) ----------
    let red_mvp = game.mvp.red.data.total_score - game.mvp.red.data.penalty_score;
    let blue_mvp = game.mvp.blue.data.total_score - game.mvp.blue.data.penalty_score;

    let total_red = game.teams.0.iter()
        .filter(|t| matches!(t.station, Stations::Red1 | Stations::Red2 | Stations::Red3))
        .count();

    let total_blue = game.teams.0.iter()
        .filter(|t| matches!(t.station, Stations::Blue1 | Stations::Blue2 | Stations::Blue3))
        .count();

    let check_alliance = |trusted: &Vec<i32>, total: usize, sum: i32, mvp: i32| {
        let unknowns = total - trusted.len();
        (unknowns == 0 && sum != mvp) || (unknowns == 1 && sum > mvp)
    };

    if check_alliance(&trusted_red, total_red, red_sum, red_mvp) {
        teams_to_redo.extend(&trusted_red);
        for team in &game.teams.0 {
            if trusted_red.contains(&team.id) {
                for s in team.scouters.iter().copied() {
                    if failed_ids.insert(s.id) {
                        fails.push(s);
                    }
                }
            }
        }
    }

    if check_alliance(&trusted_blue, total_blue, blue_sum, blue_mvp) {
        teams_to_redo.extend(&trusted_blue);
        for team in &game.teams.0 {
            if trusted_blue.contains(&team.id) {
                for s in team.scouters.iter().copied() {
                    if failed_ids.insert(s.id) {
                        fails.push(s);
                    }
                }
            }
        }
    }

    teams_to_redo.sort_unstable();
    teams_to_redo.dedup();

    forgive.retain(|s| !failed_ids.contains(&s.id));

    // ---------- GLOBAL INVARIANTS ----------
    debug_assert!(
        forgive.iter().all(|s| !failed_ids.contains(&s.id)),
        "A scouter cannot be both forgiven and failed"
    );

    debug_assert!(
        teams_to_redo.iter().all(|id| {
            game.teams.0.iter().any(|t| t.id == *id)
        }),
        "Redo list contains unknown team IDs"
    );

    Ok(CheckFailerReturn {
        game_number: game.match_id,
        teams_to_redo,
        scouts_to_forgive: forgive,
        reasons: fails,
        winner_teams,
    })
}

use crate::{backenddb::game, snowgrave::datatypes::{self, GameFull, MvpPairFull, MvpUpcomingFull, ScoutGameFull, ScoutMatch, ScoutMatchFull}};


pub enum FilledCheck {
    Filled(datatypes::GameFull),
    NotFilled
}

fn check_fill_scout(scouts: &Vec<ScoutMatch>) -> Result<Vec<ScoutMatchFull>, FilledCheck> {
    let mut new: Vec<ScoutMatchFull> = Vec::new();
    for scout in scouts {
        if let Some(data) = &scout.data {
            new.push(ScoutMatchFull {
                name: scout.name,
                station: scout.station,
                done: scout.done,
                is_redo: scout.is_redo,
                team: scout.team,
                data: data.clone(),
                id: scout.id,
            });
        } else {
            return Err(FilledCheck::NotFilled);
        }
    }
    return Ok(new);
}

pub fn check_if_filled(game: datatypes::Game) -> FilledCheck {
    let mvp_blue: MvpUpcomingFull;
    if let Some(mvp_b) = &game.mvp.blue.data {
        mvp_blue = MvpUpcomingFull {
            id: game.mvp.blue.id,
            name: game.mvp.blue.name,
            data: mvp_b.clone()
        };
    } else {
        return FilledCheck::NotFilled;
    }

    let mvp_red: MvpUpcomingFull;
    if let Some(mvp_r) = &game.mvp.red.data {
        mvp_red = MvpUpcomingFull {
            id: game.mvp.red.id,
            name: game.mvp.red.name,
            data: mvp_r.clone()
        };
    } else {
        return FilledCheck::NotFilled;
    }

    let red_1_full = match check_fill_scout(&game.scout.red_1) {
        Ok(a) => a,
        Err(a) => {
            return a;
        },
    };

    let red_2_full = match check_fill_scout(&game.scout.red_2) {
        Ok(a) => a,
        Err(a) => {
            return a;
        },
    };

    let red_3_full = match check_fill_scout(&game.scout.red_3) {
        Ok(a) => a,
        Err(a) => {
            return a;
        },
    };

    let blue_1_full = match check_fill_scout(&game.scout.blue_1) {
        Ok(a) => a,
        Err(a) => {
            return a;
        },
    };

    let blue_2_full = match check_fill_scout(&game.scout.blue_2) {
        Ok(a) => a,
        Err(a) => {
            return a;
        },
    };

    let blue_3_full = match check_fill_scout(&game.scout.blue_3) {
        Ok(a) => a,
        Err(a) => {
            return a;
        },
    };



    return FilledCheck::Filled(GameFull {
        event_code: game.event_code,
        match_id: game.match_id,
        set: game.set,
        tournament_level: game.tournament_level,
        scout: ScoutGameFull {
            red_1: red_1_full,
            red_2: red_2_full,
            red_3: red_3_full,
            blue_1: blue_1_full,
            blue_2: blue_2_full,
            blue_3: blue_3_full,
        },
        mvp: MvpPairFull {
            red: mvp_red,
            blue: mvp_blue,
        },
    });
}
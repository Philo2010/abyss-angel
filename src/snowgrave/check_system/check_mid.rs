use crate::{SETTINGS, backenddb::game::game_dispatch, snowgrave::check_system::precheck::PreCheckGame};


pub struct Range {
    pub start: f32,
    pub end: f32,
}

impl Range {
    pub fn new(main_point: f32) -> Range {
        Range {
            start: main_point - super::SCORE_RANGE_TOLERANCE,
            end: main_point + super::SCORE_RANGE_TOLERANCE,
        }
    }

    // true = fits within range, false = outside range
    pub fn check(&self, point: f32) -> bool {
        point >= self.start && point <= self.end  // Fix: was >= end && <= start (always false)
    }
}

#[derive(Debug)]
pub enum CheckReturn {  // Fix: made pub so it's usable outside this module
    Red,
    RedAndBlue,
    Blue,
    NoFail,
}

pub fn check_mid(game: &PreCheckGame) -> CheckReturn {
    let true_score_red_mvp = (game.total_score_red - game.penalty_score_red) as f32;
    let true_score_blue_mvp = (game.total_score_blue - game.penalty_score_blue) as f32;

    let game_func = game_dispatch(SETTINGS.year);

    let red_total = (game_func.get_scores(&game.red1.game).total_score
        + game_func.get_scores(&game.red2.game).total_score
        + game_func.get_scores(&game.red3.game).total_score) as f32;  // Fix: cast to f32

    let blue_total = (game_func.get_scores(&game.blue1.game).total_score
        + game_func.get_scores(&game.blue2.game).total_score
        + game_func.get_scores(&game.blue3.game).total_score) as f32;  // Fix: cast to f32

    let red_range = Range::new(true_score_red_mvp);
    let blue_range = Range::new(true_score_blue_mvp);

    eprintln!(
        "[check_mid] red: scout_sum={:.0} mvp={:.0} window=[{:.0},{:.0}] ok={} | blue: scout_sum={:.0} mvp={:.0} window=[{:.0},{:.0}] ok={}",
        red_total, true_score_red_mvp, red_range.start, red_range.end, red_range.check(red_total),
        blue_total, true_score_blue_mvp, blue_range.start, blue_range.end, blue_range.check(blue_total),
    );

    if !red_range.check(red_total) {
        if !blue_range.check(blue_total) {
            return CheckReturn::RedAndBlue;
        }
        return CheckReturn::Red;
    }

    if !blue_range.check(blue_total) {
        return CheckReturn::Blue;
    }

    CheckReturn::NoFail
}
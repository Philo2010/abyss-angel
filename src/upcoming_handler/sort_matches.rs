use std::cmp::Ordering;

use crate::upcoming_handler::{self, upcoming_game};




pub fn sort_matches(matches: &mut Vec<(upcoming_game::Model, Vec<upcoming_handler::upcoming_team::Model>)>) {
    matches.sort_by(|a, b| 

        if a.0.tournament_level == b.0.tournament_level {
            if a.0.set_number == b.0.set_number {
                a.0.match_number.cmp(&b.0.match_number)
            } else {
                a.0.set_number.cmp(&b.0.set_number)
            }
        } else if a.0.tournament_level == "Qualification" {
            Ordering::Less
        } else if b.0.tournament_level == "Qualification" {
            Ordering::Greater
        } else {
            a.0.tournament_level.cmp(&b.0.tournament_level)
        }
    );

    matches.iter_mut().for_each(|a|
      a.1.sort_by( |b, c|
        b.station.cmp(&c.station)
      )
    );
}
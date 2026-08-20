//! Prescout: admin-entered game data for a match that was never queued.
//!
//! Everything else in snowgrave reaches final storage through the pipeline in
//! `check_system` — queue the match, assign scouts, collect all six robots plus
//! both MVPs, reconcile, then publish. Prescout deliberately skips all of that:
//! an admin transcribing a team off another event's webcast has no upcoming_game,
//! no scout assignments and no opposing alliance to reconcile against.
//!
//! The row it writes is a normal game row (same header table, same year-specific
//! table, server-computed scores) marked with `is_prescout`, which keeps it out
//! of every average, graph and DPDG calculation. See `game::prescout_filter`.

use schemars::JsonSchema;
use sea_orm::{DatabaseConnection, DbErr};
use serde::Deserialize;
use uuid::Uuid;

use crate::backenddb::game::{GamesInserts, GamesInsertsSpecific, HeaderInsert, game_dispatch, insert_game};
use crate::entity::sea_orm_active_enums::{Stations, TournamentLevels};
use crate::SETTINGS;

#[derive(Deserialize, JsonSchema)]
pub struct PrescoutInsert {
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    /// Defence rating, 0-5, same scale as a normal scout submission.
    pub defence: f32,
    pub comment: String,
    pub game: GamesInsertsSpecific,
}

/// Store one prescout game. Returns the new header id.
pub async fn insert_prescout(user: Uuid, data: PrescoutInsert, db: &DatabaseConnection) -> Result<i32, DbErr> {
    let mut game = data.game;

    // DPDG is the opposing alliance's event averages minus their scores for this
    // match, so it is meaningless without the other five robots — and nothing
    // ever recomputes it for a prescout row. Null it out rather than trusting
    // whatever the client sent in the game payload.
    game_dispatch(SETTINGS.year).set_dpdg(&mut game, None, None);

    let insert = GamesInserts {
        header: HeaderInsert {
            user: vec![user],
            team: data.team,
            is_ab_team: data.is_ab_team,
            match_id: data.match_id,
            set: data.set,
            defence: data.defence,
            event_code: data.event_code,
            tournament_level: data.tournament_level,
            station: data.station,
            // There is no MVP scouter for a match that was never queued.
            is_mvp: false,
            comment: data.comment,
            is_prescout: true,
        },
        game,
    };

    insert_game(&insert, db).await
}

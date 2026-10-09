//! Batch DPDG recompute — the migration/backfill entry point.
//!
//! [`run`] recomputes and stores the split DPDG columns for every finalized game
//! of a year using the **latest** event averages. It shares the exact statement
//! and semantics of the live write-path refresh
//! (`snowgrave::check_system::dpdg_refresh::refresh_event_dpdg`), so a migration
//! backfills the same values the live pipeline stores — no as-of-match cutoff.
//!
//! A row's DPDG columns are `NULL` when the game was not the main defender
//! (`defence_main = false`), and also for a main defender whose match could not
//! be scored (no opposing rows, or a `Bot` target that is not in the match).
//!
//! The value is split by defence target: a `Bot` target writes the two *team*
//! columns (`dpdg_team` / `dpdg_team_raw`); an `Alliance` target writes the two
//! *alliance* columns (`dpdg_alliance` / `dpdg_alliance_raw`). Exactly one side
//! is populated per row.

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};

use crate::entity::genertic_header;
use crate::snowgrave::check_system::dpdg_refresh::refresh_event_dpdg;

/// Recompute and store the split DPDG values for every finalized game of
/// `year_id`.
///
/// Returns `(updated, nulled)`: how many header rows were written, and how many
/// main-defender rows were left with no DPDG (their match could not be scored).
pub async fn run(
    db: &DatabaseConnection,
    year_id: i32,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    // Every event that has finalized (non-prescout) data this year. Prescout
    // rows never carry DPDG, so they are excluded from the sweep.
    let events: Vec<(String,)> = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(year_id))
        .filter(genertic_header::Column::IsPrescout.eq(false))
        .select_only()
        .column(genertic_header::Column::EventCode)
        .group_by(genertic_header::Column::EventCode)
        .into_tuple()
        .all(db)
        .await?;

    // Refresh each event from its latest averages (one set-based statement each).
    let mut updated = 0usize;
    for (event,) in &events {
        updated += refresh_event_dpdg(event, year_id, db).await? as usize;
    }

    // Main defenders still without a value after the sweep — matches that could
    // not be scored.
    let nulled = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(year_id))
        .filter(genertic_header::Column::IsPrescout.eq(false))
        .filter(genertic_header::Column::DefenceMain.eq(true))
        .filter(genertic_header::Column::DpdgTeam.is_null())
        .filter(genertic_header::Column::DpdgAlliance.is_null())
        .all(db)
        .await?
        .len();

    println!(
        "recalc_dpdg: refreshed {} event(s), {} row(s) written, {} defender(s) left NULL",
        events.len(),
        updated,
        nulled
    );

    Ok((updated, nulled))
}

//! Statically recompute and store the DPDG value for every stored game.
//!
//! DPDG is normally stamped during the live check phase (`check_bind.rs`,
//! `stamp_dpdg`) when all six games of a match are present together. [`run`] does
//! the same calculation for every stored game at once by grouping the finalized
//! game headers back up into matches.
//!
//! A row's `dpdg` / `dpdg_raw` is `NULL` **iff the game was not the main
//! defender** (`defence_main = false`). For a main defender the value is written,
//! except when this pass cannot assemble a complete six-robot match for the row
//! (fewer than six headers share the match key, the team averages fail, or a
//! `Bot` target is not actually in the match) — those are left `NULL` too.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Local};
use sea_orm::sea_query::Alias;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, QueryFilter,
    QuerySelect,
};

use crate::backenddb::game::{DefenceTarget, prescout_filter};
use crate::entity::{
    genertic_header,
    sea_orm_active_enums::{Stations, TournamentLevels},
};

fn same_alliance(a: Stations, b: Stations) -> bool {
    matches!(a, Stations::Red1 | Stations::Red2 | Stations::Red3)
        == matches!(b, Stations::Red1 | Stations::Red2 | Stations::Red3)
}

fn station_rank(s: Stations) -> u8 {
    match s {
        Stations::Red1 => 0,
        Stations::Red2 => 1,
        Stations::Red3 => 2,
        Stations::Blue1 => 3,
        Stations::Blue2 => 4,
        Stations::Blue3 => 5,
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct MatchKey {
    event_code: String,
    match_id: i32,
    set: i32,
    tournament_level: String,
}

fn level_str(l: TournamentLevels) -> String {
    match l {
        TournamentLevels::QualificationMatch => "qualification_match".to_string(),
        TournamentLevels::Quarterfinal => "quarterfinal".to_string(),
        TournamentLevels::Semifinal => "semifinal".to_string(),
        TournamentLevels::Final => "final".to_string(),
    }
}

/// Event-average total score per `(team, is_ab_team)`, mirroring
/// `check_bind::team_avg_total_scores`.
///
/// Only headers created strictly before `before` are averaged, so the DPDG for a
/// match is computed from data that already existed when that match was scouted
/// rather than from the whole event's history.
async fn team_avg_total_scores(
    event_code: &str,
    year_id: i32,
    before: DateTime<Local>,
    db: &DatabaseConnection,
) -> Result<HashMap<(i32, bool), f32>, sea_orm::DbErr> {
    let rows: Vec<(f64, i32, bool)> = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(year_id))
        .filter(genertic_header::Column::EventCode.eq(event_code))
        .filter(genertic_header::Column::CreatedAt.lt(before))
        .filter(prescout_filter(false))
        .select_only()
        .column_as(genertic_header::Column::TotalScore.avg().cast_as(Alias::new("FLOAT8")), "total_score")
        .column(genertic_header::Column::Team)
        .column(genertic_header::Column::IsAbTeam)
        .group_by(genertic_header::Column::Team)
        .group_by(genertic_header::Column::IsAbTeam)
        .into_tuple()
        .all(db)
        .await?;
    Ok(rows.into_iter().map(|(score, team, is_ab)| ((team, is_ab), score as f32)).collect())
}

/// Write the DPDG values into one generic header row.
async fn update_dpdg(
    header_id: i32,
    dpdg: Option<f32>,
    dpdg_raw: Option<f32>,
    db: &DatabaseConnection,
) -> Result<(), sea_orm::DbErr> {
    use sea_orm::ActiveModelTrait;
    let row = genertic_header::Entity::find_by_id(header_id).one(db).await?
        .ok_or_else(|| sea_orm::DbErr::Custom(format!("genertic_header row {header_id} not found")))?;
    let mut am = genertic_header::ActiveModel::from(row);
    am.dpdg = Set(dpdg);
    am.dpdg_raw = Set(dpdg_raw);
    am.update(db).await?;
    Ok(())
}

/// Recompute and store `dpdg` / `dpdg_raw` for every finalized game of `year_id`.
///
/// Returns `(updated, nulled)`: how many header rows were written, and how many
/// individual rows were left / set `NULL` because their match could not be scored.
pub async fn run(
    db: &DatabaseConnection,
    year_id: i32,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    // Load every finalized game header for the year. Prescout rows are excluded:
    // they have no opposing alliance, and letting them into the grouping below
    // would break the "exactly 6 headers per match" check for any real match
    // they happen to share a key with.
    let headers = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(year_id))
        .filter(prescout_filter(false))
        .all(db)
        .await?;
    println!("loaded {} finalized game headers", headers.len());

    // Group by full match key (event, match, set, level).
    let mut matches: HashMap<MatchKey, Vec<genertic_header::Model>> = HashMap::new();
    for h in &headers {
        matches.entry(MatchKey {
            event_code: h.event_code.clone(),
            match_id: h.match_id,
            set: h.set,
            tournament_level: level_str(h.tournament_level),
        })
        .or_default()
        .push(h.clone());
    }

    let mut updated = 0usize;
    let mut nulled = 0usize;

    for (key, mut group) in matches {
        // A complete match needs exactly one header on each of the 6 stations.
        let n_stations = group.iter().map(|h| h.station).collect::<HashSet<_>>().len();
        if group.len() != 6 || n_stations != 6 {
            println!("match {key:?}: incomplete (entries={}) -> DPDG kept NULL", group.len());
            nulled += group.len();
            continue;
        }

        // Deterministic station ordering red1..blue3.
        group.sort_by_key(|h| station_rank(h.station));

        // Baseline averages only include data that already existed when this
        // match was scouted. Use the earliest header in the group as the match
        // time so none of this match's own six headers leak into the average.
        let cutoff = group.iter().map(|h| h.created_at).min().expect("group has 6 headers");

        let team_avg = match team_avg_total_scores(&key.event_code, year_id, cutoff, db).await {
            Ok(a) => a,
            Err(e) => {
                println!("[{}] failed computing team averages: {}", key.event_code, e);
                nulled += group.len();
                continue;
            }
        };

        let all_scores: Vec<f32> = group.iter().map(|h| h.total_score).collect();
        let all_station: Vec<Stations> = group.iter().map(|h| h.station).collect();
        let all_team: Vec<i32> = group.iter().map(|h| h.team).collect();
        let all_ab: Vec<bool> = group.iter().map(|h| h.is_ab_team).collect();

        for (i, header) in group.iter().enumerate() {
            // Mirror stamp_dpdg: only the main defender gets a value, in both
            // percentage and raw-point form, scoped to whatever it was defending.
            let (dpdg, dpdg_raw) = if !header.defence_main {
                (None, None)
            } else {
                let opponents = || all_station.iter().enumerate()
                    .filter(|(j, _)| !same_alliance(all_station[*j], all_station[i]))
                    .filter(|(j, _)| match header.defence_target {
                        DefenceTarget::Alliance => true,
                        DefenceTarget::Bot(bot) => bot.number == all_team[*j] && bot.is_ab_team == all_ab[*j],
                    });

                let percent: f32 = opponents()
                    .map(|(j, _)| {
                        let avg = team_avg.get(&(all_team[j], all_ab[j])).copied().unwrap_or(0.0);
                        if avg == 0.0 {
                            0.0
                        } else {
                            (avg - all_scores[j]) / avg * 100.0
                        }
                    })
                    .sum();

                let raw: f32 = opponents()
                    .map(|(j, _)| {
                        let avg = team_avg.get(&(all_team[j], all_ab[j])).copied().unwrap_or(0.0);
                        if avg == 0.0 {
                            0.0
                        } else {
                            avg - all_scores[j]
                        }
                    })
                    .sum();

                if opponents().next().is_none() {
                    // Targeted bot isn't in this match — the metric is meaningless.
                    println!("[{}] header_id={} targets a bot outside this match -> DPDG set NULL", key.event_code, header.id);
                    nulled += 1;
                    (None, None)
                } else {
                    (Some(percent), Some(raw))
                }
            };

            match update_dpdg(header.id, dpdg, dpdg_raw, db).await {
                Ok(_) => updated += 1,
                Err(e) => println!("[{}] failed to write dpdg for header_id={} : {}", key.event_code, header.id, e),
            }
        }
    }

    Ok((updated, nulled))
}

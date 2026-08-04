//! Statically recompute and store the DPDG value for every stored game.
//!
//! DPDG is normally stamped during the live check phase (`check_bind.rs`, `stamp_dpdg`)
//! when all six games of a match are present together. This binary does the same
//! calculation for all stored games at once by grouping the finalized game headers
//! back up into matches. If the data needed for a match can't be found (fewer than
//! six games, uncertain averages, etc.) the DPDG is left/kept as `NULL`.

#[macro_use] extern crate rocket;

#[path = "../sexymac.rs"]
mod sexymac;
#[path = "../setting/mod.rs"]
mod setting;
#[path = "../frontend/mod.rs"]
mod frontend;
#[path = "../auth/mod.rs"]
mod auth;
#[path = "../pit/mod.rs"]
mod pit;
#[path = "../entity/mod.rs"]
mod entity;
#[path = "../backenddb/mod.rs"]
mod backenddb;
#[path = "../scoutwarn/mod.rs"]
mod scoutwarn;
#[path = "../snowgrave/mod.rs"]
mod snowgrave;

use rocket::tokio;
use std::collections::HashMap;

use sea_orm::sea_query::Alias;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, QueryFilter, QuerySelect};

use crate::backenddb::game::{GamesFullSpecific, game_dispatch};
use crate::entity::{genertic_header, sea_orm_active_enums::{Stations, TournamentLevels}};

const SETTINGS: crate::setting::Settings = crate::setting::Settings {
    year: 2026,
    bcrypt: 12,
    db_path: "postgres://philipbedrosian@localhost/testdb",
    blue_api_key: "fZ2lDqVUFVvi4yyXXNZv604p1v6sjKAx6mEQlDiPGQp0KOfVinntdfp8E8My5YSj"
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

/// Event-average total score per (team, is_ab_team), mirroring `team_avg_total_scores`.
async fn team_avg_total_scores(
    event_code: &str,
    year_id: i32,
    db: &DatabaseConnection,
) -> Result<HashMap<(i32, bool), f32>, sea_orm::DbErr> {
    let rows: Vec<(f64, i32, bool)> = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(year_id))
        .filter(genertic_header::Column::EventCode.eq(event_code))
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

/// Is this stored game the main defender for its year?
fn is_main_defender(game: &GamesFullSpecific) -> bool {
    match game {
        GamesFullSpecific::RebuiltGame(model) => model.defence_main,
        #[allow(unreachable_patterns)]
        _ => false,
    }
}

#[rocket::tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = sea_orm::Database::connect(SETTINGS.db_path).await?;
    let model = game_dispatch(SETTINGS.year);
    let year_id = model.get_year_id();

    // Load every finalized game header for the current year.
    let headers = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(year_id))
        .all(&db)
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
        let n_stations = group.iter().map(|h| h.station).collect::<std::collections::HashSet<_>>().len();
        if group.len() != 6 || n_stations != 6 {
            println!("match {key:?}: incomplete (entries={}) -> DPDG kept NULL", group.len());
            nulled += group.len();
            continue;
        }

        // Deterministic station ordering red1..blue3.
        group.sort_by_key(|h| station_rank(h.station));

        let team_avg = match team_avg_total_scores(&key.event_code, year_id, &db).await {
            Ok(a) => a,
            Err(e) => {
                println!("[{}] failed computing team averages: {}", key.event_code, e);
                nulled += group.len();
                continue;
            }
        };

        // Load each game's scoring/defence data from its year-specific table.
        let game_ids: Vec<i32> = group.iter().map(|h| h.game_id).collect();
        let fulls = match model.get_full_matches(game_ids.clone(), &db).await {
            Ok(f) => f,
            Err(e) => {
                println!("[{}] failed loading game data: {}", key.event_code, e);
                nulled += group.len();
                continue;
            }
        };

        if fulls.len() != group.len() {
            println!("[{}] could not resolve all full games — DPDG kept NULL", key.event_code);
            nulled += group.len();
            continue;
        }

        let all_scores: Vec<f32> = group.iter().map(|h| h.total_score).collect();
        let all_station: Vec<Stations> = group.iter().map(|h| h.station).collect();
        let all_team: Vec<i32> = group.iter().map(|h| h.team).collect();
        let all_ab: Vec<bool> = group.iter().map(|h| h.is_ab_team).collect();

        for (i, (header, full)) in group.iter().zip(fulls.iter()).enumerate() {
            // Mirror stamp_dpdg: only the main defender gets a value.
            let dpdg = if is_main_defender(full) {
                let value: f32 = all_station.iter().enumerate()
                    .filter(|(j, _)| !same_alliance(all_station[*j], all_station[i]))
                    .map(|(j, _)| {
                        let avg = team_avg.get(&(all_team[j], all_ab[j])).copied().unwrap_or(0.0);
                        if avg == 0.0 {
                            0.0
                        } else {
                            (avg - all_scores[j]) / avg * 100.0
                        }
                    })
                    .sum();
                Some(value)
            } else {
                None
            };

            // Stamp into the game-specific table.
            match update_dpdg(header.game_id, dpdg, &db).await {
                Ok(_) => {
                    updated += 1;
                }
                Err(e) => {
                    println!("[{}] failed to write dpdg for game_id={} : {}", key.event_code, header.game_id, e);
                }
            }
        }
    }

    println!("done. updated={updated} nulled={nulled}");
    Ok(())
}

/// Write the DPDG value into the year-specific game row (rebuilt / 2026).
async fn update_dpdg(
    game_id: i32,
    dpdg: Option<f32>,
    db: &DatabaseConnection,
) -> Result<(), sea_orm::DbErr> {
    use sea_orm::ActiveModelTrait;
    let row = crate::backenddb::entrys::rebuilt::Entity::find_by_id(game_id).one(db).await?
        .ok_or_else(|| sea_orm::DbErr::Custom(format!("rebuilt_game row {game_id} not found")))?;
    let mut am = crate::backenddb::entrys::rebuilt::ActiveModel::from(row);
    am.dpdg = Set(dpdg);
    am.update(db).await?;
    Ok(())
}
//! One-shot migration: brings a database created by the code at commit `7410465`
//! ("align migration's pick_list unique constraint name with schema-sync") up to
//! the current SeaORM entity schema.
//!
//! What changed since `7410465`:
//!   * `pick_list` gained `order_defence`, `order_offence`, `order_general`
//!     (all nullable `integer`).
//!
//! New rows created by the current `get.rs` automatically pull TBA rankings as
//! their default order. This migration backfills existing rows the same way,
//! then renumbers each category 1..N.
//!
//! Idempotent: every DDL statement is guarded with `IF NOT EXISTS`.
//! Run this before starting the new server against an old DB.
//!
//! Usage: `cargo run --bin migrate_7410465 [-- <postgres-url>]`
//! (defaults to `SETTINGS.db_path`).

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
#[path = "../pick_list/mod.rs"]
mod pick_list;

use std::collections::HashMap;

use rocket::tokio;
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement, TransactionTrait};
use crate::snowgrave::blue;

const SETTINGS: crate::setting::Settings = crate::setting::Settings {
    year: 2026,
    bcrypt: 12,
    db_path: "postgres://philipbedrosian@localhost/testdb",
    blue_api_key: "fZ2lDqVUFVvi4yyXXNZv604p1v6sjKAx6mEQlDiPGQp0KOfVinntdfp8E8My5YSj"
};

const SCRIPT: &str = r#"
ALTER TABLE pick_list
  ADD COLUMN IF NOT EXISTS order_defence integer,
  ADD COLUMN IF NOT EXISTS order_offence integer,
  ADD COLUMN IF NOT EXISTS order_general integer;
"#;

#[rocket::tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::args().nth(1).unwrap_or_else(|| SETTINGS.db_path.to_string());
    let db = Database::connect(&db_url).await?;

    // 1. Add columns (idempotent).
    let txn = db.begin().await?;
    txn.execute_unprepared(SCRIPT).await?;
    txn.commit().await?;
    println!("columns added");

    // 2. Find distinct event codes that have rows needing backfill.
    let rows: Vec<sea_orm::QueryResult> = db
        .query_all_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT DISTINCT event_code FROM pick_list WHERE order_defence IS NULL OR order_offence IS NULL OR order_general IS NULL",
        ))
        .await?;

    let event_codes: Vec<String> = rows
        .into_iter()
        .filter_map(|r| r.try_get_by_index::<String>(0).ok())
        .collect();

    if event_codes.is_empty() {
        println!("no rows need backfill");
        println!("migration complete");
        return Ok(());
    }

    println!("backfilling orders for {} event(s)", event_codes.len());

    let client = reqwest::Client::new();

    for event_code in &event_codes {
        // 2a. Pull TBA rankings.
        let rankings = blue::get_ranking_from_blue(&client, event_code).await.unwrap_or_else(|_| {
            blue::EventRankingNice { rankings: Vec::new() }
        });
        let rank_map: HashMap<(i32, bool), i32> = rankings.rankings.iter()
            .map(|r| ((r.team_key.number, r.team_key.is_ab_team), r.rank))
            .collect();
        let default_rank = rankings.rankings.len() as i32 + 1;

        // 2b. Load all pick_list rows for this event that still have a NULL order.
        let rows: Vec<sea_orm::QueryResult> = db
            .query_all_raw(Statement::from_string(
                DbBackend::Postgres,
                format!("SELECT id, team, team_is_ab_team FROM pick_list WHERE event_code = '{event_code}' AND (order_defence IS NULL OR order_offence IS NULL OR order_general IS NULL)"),
            ))
            .await?;

        for r in &rows {
            let id: i32 = r.try_get_by_index::<i32>(0)?;
            let team: i32 = r.try_get_by_index::<i32>(1)?;
            let is_ab: bool = r.try_get_by_index::<bool>(2)?;
            let order = rank_map.get(&(team, is_ab)).copied().unwrap_or(default_rank);

            db.execute_unprepared(&format!(
                "UPDATE pick_list SET
                   order_defence = COALESCE(order_defence, {order}),
                   order_offence = COALESCE(order_offence, {order}),
                   order_general = COALESCE(order_general, {order})
                 WHERE id = {id}",
                order = order,
                id = id,
            )).await?;
        }

        // 2c. Renumber each category 1..N.
        for col in &["order_defence", "order_offence", "order_general"] {
            db.execute_unprepared(&format!(
                "UPDATE pick_list SET {col} = sub.new_ord FROM (
                   SELECT id, row_number() OVER (ORDER BY {col} NULLS LAST, team) AS new_ord
                   FROM pick_list WHERE event_code = '{event_code}'
                 ) sub WHERE pick_list.id = sub.id AND pick_list.{col} IS DISTINCT FROM sub.new_ord",
                col = col,
                event_code = event_code,
            )).await?;
        }

        println!("  backfilled event {}", event_code);
    }

    println!("migration complete");
    Ok(())
}
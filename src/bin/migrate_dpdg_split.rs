//! One-shot migration: moves a database from the old 2-column DPDG model
//! (`genertic_header.dpdg` / `dpdg_raw`) to the new 4-column split model
//! (`dpdg_team` / `dpdg_team_raw` / `dpdg_alliance` / `dpdg_alliance_raw`).
//!
//! What changed:
//!   * DPDG is now split by defence target. A `Bot` target is scored against the
//!     single targeted robot and written to the two *team* columns; an
//!     `Alliance` target is averaged over the whole opposing alliance and written
//!     to the two *alliance* columns.
//!   * `genertic_header.dpdg` / `dpdg_raw` are dropped.
//!
//! Because a stored `dpdg` value conflates the team and alliance contributions,
//! it cannot be split in place — the four new columns are backfilled by
//! recomputing from stored `total_score` and the event team averages, exactly as
//! the live pipeline does (see `backenddb::recalc_dpdg`).
//!
//! Idempotent: every DDL statement is guarded with `IF [NOT] EXISTS`.
//! **Run this before starting the new server against an old DB** — otherwise the
//! startup schema-sync drops `dpdg` / `dpdg_raw` before this can read them, and
//! adds the new columns without backfilling them.
//!
//! Usage: `cargo run --bin migrate_dpdg_split [-- <postgres-url>]`
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

use rocket::tokio;
use sea_orm::{ConnectionTrait, Database, TransactionTrait};

const SETTINGS: crate::setting::Settings = crate::setting::Settings {
    year: 2026,
    bcrypt: 12,
    db_path: "postgres://philipbedrosian@localhost/testdb",
    blue_api_key: "fZ2lDqVUFVvi4yyXXNZv604p1v6sjKAx6mEQlDiPGQp0KOfVinntdfp8E8My5YSj"
};

/// DDL only: add the four split columns, drop the old pair. Postgres runs the
/// whole multi-statement string in one simple-query batch.
const SCRIPT: &str = r#"
ALTER TABLE genertic_header
  ADD COLUMN IF NOT EXISTS dpdg_team          real,
  ADD COLUMN IF NOT EXISTS dpdg_team_raw      real,
  ADD COLUMN IF NOT EXISTS dpdg_alliance      real,
  ADD COLUMN IF NOT EXISTS dpdg_alliance_raw  real;

ALTER TABLE genertic_header
  DROP COLUMN IF EXISTS dpdg,
  DROP COLUMN IF EXISTS dpdg_raw;
"#;

#[rocket::tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::args().nth(1).unwrap_or_else(|| SETTINGS.db_path.to_string());
    let db = Database::connect(&db_url).await?;

    // 1. Schema DDL, all-or-nothing.
    let txn = db.begin().await?;
    txn.execute_unprepared(SCRIPT).await?;
    txn.commit().await?;
    println!("schema migrated to split DPDG");

    // 2. Recompute the four split values for every stored game.
    let (updated, nulled) = backenddb::recalc_dpdg::run(&db, SETTINGS.year).await?;
    println!("dpdg recompute: updated={updated} nulled={nulled}");

    println!("migration complete");
    Ok(())
}

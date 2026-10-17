//! One-shot migration: adds the pit-scouting metrics columns that commit
//! `3c91963` ("added all the pimton pits") introduced on `pit_rebuilt` but that
//! never got a migration.
//!
//! What changed:
//!   * `pit_rebuilt` gained `bps` (scoring rate, in balls per second) and
//!     `ball_capasity` (on-board ball capacity), both `real` (Rust `f32`) and
//!     `NOT NULL`.
//!
//! Both columns are non-nullable with no sensible source, so existing rows are
//! backfilled with `0` via a temporary default that is dropped again below so
//! the columns end up matching the entity definition exactly.
//!
//! Idempotent: every statement is guarded with `IF [NOT] EXISTS`.
//! **Run this before starting the new server against an old DB** — otherwise
//! the startup schema-sync tries to add these `NOT NULL` columns to a populated
//! `pit_rebuilt` table and fails. This is the pit-scouting counterpart to
//! `migrate_dpdg_split`, which covered the DPDG change from the same commit.
//!
//! Usage: `cargo run --bin migrate_pit_bps_capacity [-- <postgres-url>]`
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

/// DDL only. Postgres runs the whole multi-statement string in one simple-query
/// batch. The `NOT NULL` columns get a temporary default so `ADD COLUMN`
/// backfills existing rows; the default is dropped again so the column matches
/// the entity definition exactly.
const SCRIPT: &str = r#"
ALTER TABLE pit_rebuilt
  ADD COLUMN IF NOT EXISTS bps           real NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS ball_capasity real NOT NULL DEFAULT 0;

ALTER TABLE pit_rebuilt
  ALTER COLUMN bps           DROP DEFAULT,
  ALTER COLUMN ball_capasity DROP DEFAULT;
"#;

#[rocket::tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::args().nth(1).unwrap_or_else(|| SETTINGS.db_path.to_string());
    let db = Database::connect(&db_url).await?;

    // Schema DDL, all-or-nothing.
    let txn = db.begin().await?;
    txn.execute_unprepared(SCRIPT).await?;
    txn.commit().await?;
    println!("added pit_rebuilt.bps / pit_rebuilt.ball_capasity");

    println!("migration complete");
    Ok(())
}

//! One-shot migration: brings a database created by the code at commit `7410465`
//! ("align migration's pick_list unique constraint name with schema-sync") up to
//! the current SeaORM entity schema.
//!
//! What changed since `7410465`:
//!   * `pick_list` gained `order_defence`, `order_offence`, `order_general`
//!     (all nullable `integer` — no backfill needed).
//!
//! Idempotent: every statement is guarded with `IF NOT EXISTS`.
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

use rocket::tokio;
use sea_orm::{ConnectionTrait, Database, TransactionTrait};

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

    let txn = db.begin().await?;
    txn.execute_unprepared(SCRIPT).await?;
    txn.commit().await?;

    println!("migration complete");
    Ok(())
}
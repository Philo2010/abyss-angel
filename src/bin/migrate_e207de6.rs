//! One-shot migration: brings a database created by the code at commit `e207de6`
//! ("slop2 electirc booglalooo") up to the current SeaORM entity schema, then
//! recomputes DPDG for every stored game.
//!
//! What changed since `e207de6`:
//!   * `genertic_header` gained `is_prescout`, `defence_main`, `defence_target`,
//!     `auto_time`, `dead`, `dnf`, `dpdg`, `dpdg_raw`. `defence_main` / `dead` /
//!     `dnf` were **moved off `rebuilt_game`** (the "generic header refactor",
//!     commit `6a21068`) — this migration copies those values across before the
//!     old columns are dropped.
//!   * `mid_header` gained `defence_main`, `defence_target`, `auto_time`, `dead`,
//!     `dnf` (transient in-progress rows — filled with defaults).
//!   * `users` gained `is_pick`.
//!   * `pick_list` is a new table.
//!
//! DPDG semantics: `dpdg` / `dpdg_raw` are `NULL` **iff the game was not the main
//! defender** (`defence_main = false`). For a main defender they are populated by
//! the recompute below, unless a complete six-robot match cannot be assembled for
//! that row (incomplete data, or a `Bot` target absent from the match).
//!
//! Idempotent: every statement is guarded with `IF [NOT] EXISTS`, and the DDL +
//! backfill run in one transaction. **Run this before starting the new server
//! against an old DB** — otherwise the startup schema-sync drops
//! `rebuilt_game.defence_main/dead/dnf` before this can copy them over.
//!
//! Usage: `cargo run --bin migrate_e207de6 [-- <postgres-url>]`
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

/// DDL + data backfill. `$YEAR` is substituted with `SETTINGS.year` (the
/// `game_type_id` of `rebuilt_game` rows) before execution. Postgres runs the
/// whole multi-statement string in one simple-query batch.
const SCRIPT: &str = r#"
-- genertic_header: add the new columns. The NOT NULL ones get a temporary
-- default so ADD COLUMN backfills existing rows; the default is dropped again
-- below so the column matches the entity definition exactly.
ALTER TABLE genertic_header
  ADD COLUMN IF NOT EXISTS is_prescout    boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS defence_main   boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS defence_target jsonb   NOT NULL DEFAULT '"Alliance"'::jsonb,
  ADD COLUMN IF NOT EXISTS auto_time      real    NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS dead           boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS dnf            boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS dpdg           real,
  ADD COLUMN IF NOT EXISTS dpdg_raw       real;

-- Copy defence_main / dead / dnf back from rebuilt_game (they lived there before
-- the generic header refactor). Only rebuilt_game-backed headers match. Guarded
-- so a second run (after the rebuilt_game columns are dropped below) is a no-op.
DO $do$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'rebuilt_game' AND column_name = 'defence_main'
  ) THEN
    UPDATE genertic_header h
    SET defence_main = r.defence_main,
        dead         = r.dead,
        dnf          = r.dnf
    FROM rebuilt_game r
    WHERE h.game_id = r.id
      AND h.game_type_id = $YEAR;
  END IF;
END
$do$;

-- Drop the temporary defaults.
ALTER TABLE genertic_header
  ALTER COLUMN is_prescout    DROP DEFAULT,
  ALTER COLUMN defence_main   DROP DEFAULT,
  ALTER COLUMN defence_target DROP DEFAULT,
  ALTER COLUMN auto_time      DROP DEFAULT,
  ALTER COLUMN dead           DROP DEFAULT,
  ALTER COLUMN dnf            DROP DEFAULT;

-- rebuilt_game: the moved columns are gone from the entity.
ALTER TABLE rebuilt_game
  DROP COLUMN IF EXISTS defence_main,
  DROP COLUMN IF EXISTS dead,
  DROP COLUMN IF EXISTS dnf;

-- mid_header: new transient-row columns, defaults only (no source table).
ALTER TABLE mid_header
  ADD COLUMN IF NOT EXISTS defence_main   boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS defence_target jsonb,
  ADD COLUMN IF NOT EXISTS auto_time      real    NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS dead           boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS dnf            boolean NOT NULL DEFAULT false;
ALTER TABLE mid_header
  ALTER COLUMN defence_main DROP DEFAULT,
  ALTER COLUMN auto_time    DROP DEFAULT,
  ALTER COLUMN dead         DROP DEFAULT,
  ALTER COLUMN dnf          DROP DEFAULT;

-- users: pick-list permission flag.
ALTER TABLE users
  ADD COLUMN IF NOT EXISTS is_pick boolean NOT NULL DEFAULT false;
ALTER TABLE users
  ALTER COLUMN is_pick DROP DEFAULT;

-- pick_list: brand-new table.
CREATE TABLE IF NOT EXISTS pick_list (
  id                  serial  PRIMARY KEY,
  team                integer NOT NULL,
  team_is_ab_team     boolean NOT NULL,
  event_code          varchar NOT NULL,
  is_selected_defence boolean NOT NULL,
  is_selected_offence boolean NOT NULL,
  is_selected_general boolean NOT NULL,
  CONSTRAINT pick_list_item_unique UNIQUE (team, team_is_ab_team, event_code)
);
"#;

#[rocket::tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::args().nth(1).unwrap_or_else(|| SETTINGS.db_path.to_string());
    let db = Database::connect(&db_url).await?;

    // 1. Schema DDL + cross-table backfill, all-or-nothing.
    let script = SCRIPT.replace("$YEAR", &SETTINGS.year.to_string());
    let txn = db.begin().await?;
    txn.execute_unprepared(&script).await?;
    txn.commit().await?;
    println!("schema migrated");

    // 2. Recompute dpdg / dpdg_raw for every stored game.
    let (updated, nulled) = backenddb::recalc_dpdg::run(&db, SETTINGS.year).await?;
    println!("dpdg recompute: updated={updated} nulled={nulled}");

    println!("migration complete");
    Ok(())
}

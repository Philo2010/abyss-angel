//! Statically recompute and store the DPDG value for every stored game.
//!
//! Thin wrapper over [`backenddb::recalc_dpdg::run`]; see that module for the
//! rules (in short: DPDG is `NULL` unless the game was the main defender).

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

const SETTINGS: crate::setting::Settings = crate::setting::Settings {
    year: 2026,
    bcrypt: 12,
    db_path: "postgres://philipbedrosian@localhost/testdb",
    blue_api_key: "fZ2lDqVUFVvi4yyXXNZv604p1v6sjKAx6mEQlDiPGQp0KOfVinntdfp8E8My5YSj"
};

#[rocket::tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = sea_orm::Database::connect(SETTINGS.db_path).await?;
    let (updated, nulled) = backenddb::recalc_dpdg::run(&db, SETTINGS.year).await?;
    println!("done. updated={updated} nulled={nulled}");
    Ok(())
}

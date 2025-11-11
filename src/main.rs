use rocket::tokio;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Schema, sea_query::PostgresQueryBuilder};

use crate::setting::Settings;

mod user;
mod sexymac;
mod setting;
mod frontend;

//For now, before i make a setting menu, i will hardcode values
const SETTINGS: crate::setting::Settings = Settings {
    year: 2025
};


#[tokio::main]
async fn main() {
    println!("Hello World!");
}

#[macro_use] extern crate rocket;
use rocket::{build, fs::FileServer, fs::relative};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Schema, sea_query::PostgresQueryBuilder};

use crate::setting::Settings;

mod user;
mod sexymac;
mod setting;
mod frontend;


//For now, before i make a setting menu, i will hardcode values
const SETTINGS: crate::setting::Settings = Settings {
    year: 2025,
    db_path: "postgres://localhost/testdb",
};


#[get("/")]
async fn placeholder() -> &'static str {
    "Hello!"
}


#[launch]
async fn rocket() -> _ {

    let db_conn = match sea_orm::Database::connect(SETTINGS.db_path).await {
        Ok(a) => a,
        Err(a) => {
            println!("Major issue! We were not able to connect to database, this is very funny as we were able to connect to the database before (or else you would not be seeing this)");
            println!("Err from Seaorm: {a}");
            panic!();
        },
    };



    rocket::build()
    .manage(db_conn)
    .mount("/", routes![placeholder,
    frontend::graph::graph,
    frontend::scout::scout_take])
    .mount("/", FileServer::from(relative!("static")))
}
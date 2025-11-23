#[macro_use] extern crate rocket;
use rocket::{build, fs::{FileServer, relative}, tokio::sync::RwLock};
use rocket_dyn_templates::Template;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, EntityTrait, Schema, sea_query::PostgresQueryBuilder};
use rocket::local::asynchronous::Client;

use crate::setting::{Settings};

mod user;
mod sexymac;
mod setting;
mod frontend;
mod models;
mod upcoming_handler;


//For now, before i make a setting menu, i will hardcode values
const SETTINGS: crate::setting::Settings = Settings {
    year: 2025,
    db_path: "postgres://philipbedrosian@localhost/testdb",
    blue_api_key: "{{ INSERT_API_KEY }}"
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

    let client = reqwest::Client::new();


    rocket::build()
    .manage(db_conn)
    .manage(client)
    .attach(Template::fairing())
    .mount("/", routes![placeholder,
    frontend::graph::graph,
    frontend::scout::scout_take,
    frontend::averages::averages_empty,
    frontend::averages::averages_event,
    frontend::allentry::allentry,
    frontend::search::search,
    frontend::search::search_default])
    .mount("/", FileServer::from(relative!("static")))
}
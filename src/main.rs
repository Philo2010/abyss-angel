#[macro_use] extern crate rocket;
use rocket::{Config, build, data::{ByteUnit, Limits}, fs::{FileServer, relative}, tokio::sync::RwLock};
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
mod auth;


//For now, before i make a setting menu, i will hardcode values
const SETTINGS: crate::setting::Settings = Settings {
    year: 2025,
    bcrypt: 12,
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

    let limits = Limits::default()
        .limit("form", ByteUnit::Megabyte(5));  // Note: "form" not "forms"!

    let figment = Config::figment()
        .merge(("limits", limits));


    rocket::custom(figment)
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
    frontend::search::search_default,
    upcoming_handler::query_game::queue,
    upcoming_handler::select_scouters_page::select_scouts,
    upcoming_handler::submit_scout::assign_scout,
    frontend::scout_auto::scout_auto,
    auth::create_user::create_user,
    auth::login::login,
    ])
    .mount("/", FileServer::from(relative!("static")))
}
#[macro_use] extern crate rocket;
use rocket::{Config, data::{ByteUnit, Limits}, fs::{FileServer, relative}};
use rocket_dyn_templates::Template;
use rocket_okapi::{r#gen::OpenApiGenerator, okapi::openapi3::OpenApi, openapi_get_routes_spec, openapi_get_spec, settings::OpenApiSettings};
use serde_json::to_string_pretty;

use crate::{frontend::delete::delete_scout, setting::Settings};

mod sexymac;
mod setting;
mod frontend;
mod auth;
mod pit;
mod entity;
mod backenddb;
mod snowgrave;
mod scoutwarn;

//For now, before i make a setting menu, i will hardcode values
const SETTINGS: crate::setting::Settings = Settings {
    year: 2025,
    bcrypt: 12,
    db_path: "postgres://philipbedrosian@localhost/testdb",
    blue_api_key: "{{ INSERT_API_KEY }}"
};

fn main() {
    let settings = OpenApiSettings::default();
    use crate::frontend::pit::edit::okapi_add_operation_for_edit_pit_;
    use crate::frontend::pit::get::okapi_add_operation_for_get_;
    use crate::frontend::pit::insert::okapi_add_operation_for_insert_;
    use crate::frontend::scoutwarn::forgive_warning::okapi_add_operation_for_forgive_scoutwarn_;
    use crate::frontend::scoutwarn::get_warning::okapi_add_operation_for_get_scoutwarn_;
    use crate::frontend::scoutwarn::send_warning::okapi_add_operation_for_send_scoutwarn_;
    use crate::frontend::snowgrave::find_games::okapi_add_operation_for_get_years_;
    use crate::frontend::snowgrave::queue::okapi_add_operation_for_queue_;
    use crate::frontend::averages::okapi_add_operation_for_averages_;
    use crate::frontend::delete::okapi_add_operation_for_delete_scout_;
    use crate::frontend::graph::okapi_add_operation_for_graph_;
    use crate::frontend::search::okapi_add_operation_for_search_;

    let spec = openapi_get_spec![
        settings:
        edit_pit,
        get,
        insert,
        forgive_scoutwarn,
        get_scoutwarn,
        send_scoutwarn,
        get_years,
        queue,
        averages,
        delete_scout,
        graph,
        search
    ];

    println!("{}", serde_json::to_string_pretty(&spec).unwrap());
}

/* 
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
    .mount("/", routes![
    frontend::pit::edit::edit_pit,
    frontend::pit::get::get,
    frontend::pit::insert::insert,
    frontend::scoutwarn::forgive_warning::forgive_scoutwarn,
    frontend::scoutwarn::get_warning::get_scoutwarn,
    frontend::scoutwarn::send_warning::send_scoutwarn,
    frontend::snowgrave::find_games::get_years,
    frontend::snowgrave::queue::queue,
    frontend::averages::averages,
    frontend::delete::delete_scout,
    frontend::graph::graph,
    frontend::search::search,
    ])
    .mount("/", FileServer::from(relative!("static")))
}*/
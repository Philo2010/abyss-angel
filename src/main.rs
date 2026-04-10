#[macro_use] extern crate rocket;
use rocket::{Config, data::{ByteUnit, Limits}, tokio};
use rocket_okapi::{openapi_get_spec, settings::OpenApiSettings};

use crate::setting::Settings;

mod sexymac;
mod setting;
mod frontend;
mod auth;
mod pit;
mod entity;
mod backenddb;
mod scoutwarn;
mod snowgrave;

//For now, before i make a setting menu, i will hardcode values
const SETTINGS: crate::setting::Settings = Settings {
    year: 2026,
    bcrypt: 12,
    db_path: "postgres://philipbedrosian@localhost/testdb",
    blue_api_key: "fZ2lDqVUFVvi4yyXXNZv604p1v6sjKAx6mEQlDiPGQp0KOfVinntdfp8E8My5YSj"
};



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

    match db_conn.get_schema_registry("abyss-angel::entity").sync(&db_conn).await {
        Ok(_) => {
            
        },
        Err(a) => {
            let strhe = a.to_string();
            println!("{strhe}");
            panic!()
        },
    };

    match db_conn.get_schema_registry("abyss-angel::pit::entrys").sync(&db_conn).await {
        Ok(_) => {
            
        },
        Err(a) => {
            let strhe = a.to_string();
            println!("{strhe}");
            panic!()
        },
    };

    match db_conn.get_schema_registry("abyss-angel::backenddb::entrys").sync(&db_conn).await {
        Ok(_) => {
            
        },
        Err(a) => {
            let strhe = a.to_string();
            println!("{strhe}");
            panic!()
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
    .mount("/", routes![
    frontend::pit::edit::edit_pit,
    frontend::pit::get::get,
    frontend::pit::insert::insert,
    frontend::pit::get_all_pits::pit_get_all,
    frontend::pit::get_for_pit_scouter::get_for_scout,
    frontend::scoutwarn::forgive_warning::forgive_scoutwarn,
    frontend::scoutwarn::get_warning::get_scoutwarn,
    frontend::scoutwarn::send_warning::send_scoutwarn,
    frontend::averages::averages,
    frontend::delete::delete_scout,
    frontend::graph::graph,
    frontend::search::search,
    auth::login::login,
    auth::create_user::create_user,
    auth::check_status::check_status,
    auth::create_user::create_user_front,
    frontend::snowgrave::find_games::get_years,
    frontend::snowgrave::mvp_insert::mvp_insert,
    frontend::snowgrave::queue::queue,
    frontend::snowgrave::queue::queue_playoff,
    frontend::snowgrave::scouter_edit::scout_edit,
    frontend::snowgrave::get_teams_from_game::get_teams_from_game,
    frontend::snowgrave::scouter_insert::scout_insert,
    frontend::get_all_users::get_all_users,
    frontend::reset_password::reset_password,
    frontend::snowgrave::sub_scout::sub_scout,
    frontend::snowgrave::get_leaderboard::get_leaderboard,
    frontend::snowgrave::insert_scout::insert_scout,
    frontend::snowgrave::get_all_scouts::get_all_snowgrave,
    setting::setevent::set_event,
    setting::setevent::get_event,
    frontend::pit::assign_pit::assign_pit,
    frontend::snowgrave::delete_event::delete_event_route,
    frontend::snowgrave::manual_add_match::manual_add_match,
    frontend::snowgrave::bypass_check::bypass_check
    ])
}

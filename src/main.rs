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

/* 
fn main() {
    let settings = OpenApiSettings::default();
    use crate::frontend::pit::edit::okapi_add_operation_for_edit_pit_;
    use crate::frontend::pit::get::okapi_add_operation_for_get_;
    use crate::frontend::pit::insert::okapi_add_operation_for_insert_;
    use crate::frontend::scoutwarn::forgive_warning::okapi_add_operation_for_forgive_scoutwarn_;
    use crate::frontend::scoutwarn::get_warning::okapi_add_operation_for_get_scoutwarn_;
    use crate::frontend::scoutwarn::send_warning::okapi_add_operation_for_send_scoutwarn_;
    use crate::frontend::averages::okapi_add_operation_for_averages_;
    use crate::frontend::delete::okapi_add_operation_for_delete_scout_;
    use crate::frontend::graph::okapi_add_operation_for_graph_;
    use crate::frontend::search::okapi_add_operation_for_search_;
    use crate::auth::login::okapi_add_operation_for_login_;
    use crate::frontend::snowgrave::find_games::okapi_add_operation_for_get_years_;
    use crate::frontend::snowgrave::mvp_insert::okapi_add_operation_for_mvp_insert_;
    use crate::frontend::snowgrave::queue::okapi_add_operation_for_queue_;
    use crate::frontend::snowgrave::queue::okapi_add_operation_for_queue_playoff_;
    use crate::frontend::snowgrave::scouter_edit::okapi_add_operation_for_scout_edit_;
    use crate::frontend::snowgrave::scouter_insert::okapi_add_operation_for_scout_insert_;
    use crate::frontend::get_all_users::okapi_add_operation_for_get_all_users_;
    use crate::frontend::reset_password::okapi_add_operation_for_reset_password_;
    use crate::setting::setevent::okapi_add_operation_for_get_event_;
    use crate::setting::setevent::okapi_add_operation_for_set_event_;
    use crate::frontend::snowgrave::sub_scout::okapi_add_operation_for_sub_scout_;
    use crate::frontend::snowgrave::get_leaderboard::okapi_add_operation_for_get_leaderboard_;
    use crate::frontend::pit::get_for_pit_scouter::okapi_add_operation_for_get_for_scout_;
    use crate::frontend::pit::get_all_pits::okapi_add_operation_for_pit_get_all_;
    use crate::frontend::pit::assign_pit::okapi_add_operation_for_assign_pit_;
    use crate::frontend::snowgrave::insert_scout::okapi_add_operation_for_insert_scout_;
    use crate::frontend::snowgrave::get_all_scouts::okapi_add_operation_for_get_all_snowgrave_;
    use crate::auth::create_user::okapi_add_operation_for_create_user_front_;
    use crate::frontend::snowgrave::get_teams_from_game::okapi_add_operation_for_get_teams_from_game_;

    let spec = openapi_get_spec![
        settings:
        edit_pit,
        get,
        insert,
        forgive_scoutwarn,
        get_for_scout,
        get_scoutwarn,
        send_scoutwarn,
        averages,
        delete_scout,
        graph,
        search,
        login,
        get_years,
        mvp_insert,
        queue,
        queue_playoff,
        scout_edit,
        scout_insert,
        get_all_users,
        reset_password,
        get_all_snowgrave,
        set_event,
        get_event,
        sub_scout,
        get_leaderboard,
        pit_get_all,
        assign_pit,
        insert_scout,
        create_user_front,
        get_teams_from_game
    ];

    println!("{}", serde_json::to_string_pretty(&spec).unwrap());
}
*/

/* 
#[tokio::main]
async fn main() -> Result<(), DbErr> {
    /* */
    let db_conn = match sea_orm::Database::connect(SETTINGS.db_path).await {
        Ok(a) => a,
        Err(a) => {
            println!("Major issue! We were not able to connect to database, this is very funny as we were able to connect to the database before (or else you would not be seeing this)");
            println!("Err from Seaorm: {a}");
            panic!();
        },
    };


    let partial_game = match GamePartial::from_game_id(36, &db_conn).await {
        Ok(a) => a,
        Err(_) => {
            panic!("from game id failed");
        },
    };

    let game_data = match hydrate_game(partial_game, &db_conn).await {
        Ok(a) => a,
        Err(a) => {
            return Err(DbErr::Custom(format!("Hydrate Game failed {:?}", a)));
        },
    };

    let game_full = match game_data {
        Some(full) => full,             // all data is present
        None => {return Err(DbErr::Custom("Not all scouter done".to_string()));}
    };

    let res = match snowgrave::check::check(&game_full) {
        Ok(a) => a,
        Err(a) => {
            match a {
                CheckMatchErr::NotAllScoutersAreDone => {
                    return Err(DbErr::Custom("NotAllScoutersAreDone".to_string()));
                },
                CheckMatchErr::MvpIsNotDone => {
                    return Err(DbErr::Custom("MvpIsNotDone".to_string()));
                },
                CheckMatchErr::ThereIsNotOneScouterPerTeam => {
                    return Err(DbErr::Custom("ThereIsNotOneScouterPerTeam".to_string()));
                },
                CheckMatchErr::ThereIsNoMVP => {
                    return Err(DbErr::Custom("There is no MVP".to_string()));
                },
                CheckMatchErr::NoMatch => {
                    return Err(DbErr::Custom("No match".to_string()));
                },
                CheckMatchErr::Not6Teams => {
                    return Err(DbErr::Custom("Not6Teams".to_string()));
                },
                CheckMatchErr::DbErr(db_err) => {
                    return Err(db_err);
                },
            }
        },
    };

    println!("{:?}", res);
    check_complete::check_match(36, &db_conn).await;

    Ok(())
}*/

/* 
#[tokio::main]
async fn main() {
    let db_conn = match sea_orm::Database::connect(SETTINGS.db_path).await {
        Ok(a) => a,
        Err(a) => {
            println!("Major issue! We were not able to connect to database, this is very funny as we were able to connect to the database before (or else you would not be seeing this)");
            println!("Err from Seaorm: {a}");
            panic!();
        },
    };
    use crate::snowgrave::check_system::check_bind::check_bind;
    println!("{:?}", check_bind(3, &db_conn).await);

}
*/

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
    frontend::pit::assign_pit::assign_pit
    ])
}

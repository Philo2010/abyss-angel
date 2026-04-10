#[macro_use] extern crate rocket;

use rocket_okapi::{openapi_get_spec, settings::OpenApiSettings};

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

const SETTINGS: crate::setting::Settings = crate::setting::Settings {
    year: 2026,
    bcrypt: 12,
    db_path: "postgres://philipbedrosian@localhost/testdb",
    blue_api_key: "fZ2lDqVUFVvi4yyXXNZv604p1v6sjKAx6mEQlDiPGQp0KOfVinntdfp8E8My5YSj"
};

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
    use crate::frontend::snowgrave::manual_add_match::okapi_add_operation_for_manual_add_match_;
    use crate::frontend::snowgrave::bypass_check::okapi_add_operation_for_bypass_check_;

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
        get_teams_from_game,
        manual_add_match,
        bypass_check
    ];

    println!("{}", serde_json::to_string_pretty(&spec).unwrap());
}

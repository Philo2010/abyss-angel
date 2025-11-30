use rocket::{State, form::Form, http::CookieJar, time::error::Parse};
use reqwest::Client;
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::{auth, upcoming_handler::{blue::pull_from_blue, to_upcoming_game}};



#[derive(FromForm)]
struct QueryForm {
    game: String,
}

#[post("/queue_game", data = "<form>")]
pub async fn queue(form: Form<QueryForm>, client: &State<reqwest::Client>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Template {

    if !(auth::check::check(cookies, db).await) {
        return Template::render("error", context! {error: "Failed to auth (are you logined?)"});
    }

    let request_result = match pull_from_blue(client, &form.game).await {
        Ok(a) => a,
        Err(a) => {
            return Template::render("error", context! {error: format!("Could not connect to Blue API for this reason: {a}")});
        },
    };

    for matche in request_result {
        match to_upcoming_game::insert_upcoming_game(db, &matche, &form.game).await {
            Ok(()) => {},
            Err(to_upcoming_game::UpcomingGameError::Parse(a)) => {
                return Template::render("error", context! {error: format!("Problem parsing the team value from Blue!, here is some debug info: {a}")});
            },
            Err(to_upcoming_game::UpcomingGameError::Database(a)) => {
                return Template::render("error", context! {error: format!("Problem inserting data into database, here is some debug info: {a}")});
            },
        }
    }

    Template::render("suc", context! {message: "Done with queue!"})
}
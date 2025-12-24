use rocket::{State, http::CookieJar};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::{auth, upcoming_handler::{self, pull_event_data}};



#[get("/select_scouts/<event>")]
pub async fn select_scouts(db: &State<DatabaseConnection>, event: &str, cookies: &CookieJar<'_>) -> Template {

    if !(auth::check::check(cookies, db).await) {
        return Template::render("error", context! {error: "Failed to auth (are you logined?)"});
    }

    let mut data = match pull_event_data::pull_event_data(db.inner(), Some(event)).await {
        Ok(a) => a,
        Err(pull_event_data::EventDataErr::Data(a)) => {
            return Template::render("error", context! {error: format!("Problem getting match data: {a}")});
        },
        Err(pull_event_data::EventDataErr::Team(a)) => {
            return Template::render("error", context! {error: format!("Problem getting team data: {a}")});
        }
    };

    upcoming_handler::sort_matches::sort_matches(&mut data);

    Template::render("select_scout", context!{data: data, event_code: event})
}
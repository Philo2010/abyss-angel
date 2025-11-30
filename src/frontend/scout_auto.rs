use std::cmp::Ordering;

use rocket::State;
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::upcoming_handler::{self, pull_event_data, sort_matches, upcoming_game};

 


#[get("/select_scout")]
pub async fn scout_auto(db: &State<DatabaseConnection>) -> Template{


    let mut matches = match upcoming_handler::pull_event_data::pull_event_data(db.inner(), None).await {
        Ok(a) => a,
        Err(pull_event_data::EventDataErr::Data(a)) => {
            return Template::render("error", context! {error: format!("Problem getting match data: {a}")});
        },
        Err(pull_event_data::EventDataErr::Team(a)) => {
            return Template::render("error", context! {error: format!("Problem getting team data: {a}")});
        }
    };

    upcoming_handler::sort_matches::sort_matches(&mut matches);



    Template::render("selectscrob", context! {matches})
}
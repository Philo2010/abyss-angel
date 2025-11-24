use rocket::State;
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::upcoming_handler::{self, pull_event_data, upcoming_game};

 


#[get("/select_scout")]
pub async fn scout_auto(db: &State<DatabaseConnection>) -> Template{


    let matches = match upcoming_handler::pull_event_data::pull_event_data(db.inner(), None).await {
        Ok(a) => a,
        Err(pull_event_data::EventDataErr::Data(a)) => {
            return Template::render("error", context! {error: format!("Problem getting match data: {a}")});
        },
        Err(pull_event_data::EventDataErr::Team(a)) => {
            return Template::render("error", context! {error: format!("Problem getting team data: {a}")});
        }
    };



    Template::render("selectscrob", context! {matches})
}
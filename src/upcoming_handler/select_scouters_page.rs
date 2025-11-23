use rocket::State;
use rocket_dyn_templates::{Template, context};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityOrSelect, EntityTrait, ModelTrait, QueryFilter};

use crate::upcoming_handler::{self, pull_event_data, upcoming_game, upcoming_team};



#[get("/select_scouts/<event>")]
pub async fn select_scouts(db: &State<DatabaseConnection>, event: &str) -> Template {

    let data = match pull_event_data::pull_event_data(db.inner(), event).await {
        Ok(a) => a,
        Err(pull_event_data::EventDataErr::Data(a)) => {
            return Template::render("error", context! {error: format!("Problem getting match data: {a}")});
        },
        Err(pull_event_data::EventDataErr::Team(a)) => {
            return Template::render("error", context! {error: format!("Problem getting team data: {a}")});
        }
    };

    Template::render("select_scout", context!{data: data, event_code: event})
}
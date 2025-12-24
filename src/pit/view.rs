use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use rocket::State;

use crate::{SETTINGS, pit::pit2025::PITYEARSGETEVENTTEAM};


#[get("/pit/view/<event_code>/<team>")]
pub async fn pit_view(event_code: &str, team: i32, db: &State<DatabaseConnection>) -> Template {
    let getfunc = PITYEARSGETEVENTTEAM[&SETTINGS.year];

    let model = match getfunc(db.inner(), team, event_code).await {
        Ok(a) => {
            a
        },
        Err(a) => {
            return Template::render("error", context! {error: format!("Delete failed: {}", a.as_ref())});
        },
    };

    Template::render("pit_view", context! {data: model})
}

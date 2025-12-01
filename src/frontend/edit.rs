use rocket::{State, form::Form};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::{SETTINGS, sexymac, user::YEARSSEARCH};


#[get("/edit")]
pub async fn edit(db: &State<DatabaseConnection>) -> Template {
    let avgfunc = YEARSSEARCH[&SETTINGS.year];

    let e = match avgfunc(sexymac::get_event_default(db.inner()).await, None, None, db).await {
        Ok(a) => a,
        Err(a) => {
            let errormessage = format!("Error! Could not find avgrage: {a}");
            return Template::render("error", context! {error: errormessage});
        },
    };


    Template::render("edittable", context! {entries: e})
}

use rocket::State;
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::SETTINGS;
use crate::sexymac::get_event_default;
use crate::user::YEARSSEARCH;




#[get("/allentry")]
pub async fn allentry(db: &State<DatabaseConnection>) -> Template {
    let avgfunc = YEARSSEARCH[&SETTINGS.year];

    let e = match avgfunc(None, None, None, db).await {
        Ok(a) => a,
        Err(a) => {
            let errormessage = format!("Error! Could not find avgrage: {a}");
            return Template::render("error", context! {error: errormessage});
        },
    };


    Template::render("table", context! {entries: e})
}

#[get("/entry")]
pub async fn entry(db: &State<DatabaseConnection>) -> Template {
    let avgfunc = YEARSSEARCH[&SETTINGS.year];

    let e = match avgfunc(get_event_default(db.inner()).await, None, None, db).await {
        Ok(a) => a,
        Err(a) => {
            let errormessage = format!("Error! Could not find avgrage: {a}");
            return Template::render("error", context! {error: errormessage});
        },
    };


    Template::render("table", context! {entries: e})
}

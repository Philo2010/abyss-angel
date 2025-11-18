use std::error::Error;

use rocket::State;
use rocket::{post, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::SETTINGS;
use crate::user::{YEARSAVG, YEARSINSERT, YEARSSEARCH};




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

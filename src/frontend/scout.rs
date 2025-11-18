use std::error::Error;

use rocket::State;
use rocket::{post, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::SETTINGS;
use crate::user::YEARSINSERT;


#[post("/scout_form", data = "<body>")]
pub async fn scout_take(body: Json<Value>, db: &State<DatabaseConnection>) -> Template {
    let insrfunc = YEARSINSERT[&SETTINGS.year];
    let e = match insrfunc(db.inner(), body.into_inner()).await {
        Ok(a) => {
            Template::render("suc", context! {message: "Properly scouted"})
        },
        Err(a) => {
            let errormesage = format!("Error!: {a}");
            Template::render("error", context!{error: errormesage})
        }
    };

    e
}

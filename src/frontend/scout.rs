use std::error::Error;

use rocket::State;
use rocket::{post, serde::json::Json};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::SETTINGS;
use crate::user::YEARSINSERT;


#[post("/scout_form", data = "<body>")]
pub async fn scout_take(body: Json<Value>, db: &State<DatabaseConnection>) -> String {
    let insrfunc = YEARSINSERT[&SETTINGS.year];
    let e = match insrfunc(db.inner(), body.into_inner()).await {
        Ok(a) => {
            a
        },
        Err(a) => {
            let errormesage = format!("Error!: {a}");
            return errormesage;
        }
    };

    "Done!".to_string() //Get off my ass
}

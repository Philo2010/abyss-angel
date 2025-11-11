use std::error::Error;

use rocket::State;
use rocket::{post, serde::json::Json};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::SETTINGS;
use crate::user::{YEARSAVG, YEARSINSERT, YEARSSEARCH};




#[get("/allentry/<event>")]
async fn allentry(event: Option<&str>, db: &State<DatabaseConnection>) -> String {
    let avgfunc = YEARSSEARCH[&SETTINGS.year];
    let event_str: Option<String> = match event {
        Some(a) => Some(a.to_string()),
        None => None,
    };

    let e = match avgfunc(event_str, None, None, db).await {
        Ok(a) => a,
        Err(a) => {
            let errormessage = format!("Error! Could not find avgrage: {a}");
            return errormessage;
        },
    };


    e.to_string()
}

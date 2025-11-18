use std::error::Error;

use rocket::State;
use rocket::{post, serde::json::Json};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::SETTINGS;
use crate::user::{YEARSAVG, YEARSINSERT, YEARSSEARCH};




#[get("/allentry")]
pub async fn allentry(db: &State<DatabaseConnection>) -> String {
    let avgfunc = YEARSSEARCH[&SETTINGS.year];

    let e = match avgfunc(None, None, None, db).await {
        Ok(a) => a,
        Err(a) => {
            let errormessage = format!("Error! Could not find avgrage: {a}");
            return errormessage;
        },
    };


    e.to_string()
}

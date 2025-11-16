use std::error::Error;

use rocket::State;
use rocket::{post, serde::json::Json};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::SETTINGS;
use crate::user::{YEARSAVG, YEARSINSERT};


#[get("/averages_d")]
pub async fn averages_empty(db: &State<DatabaseConnection>) -> String {
    averages_impl(None, db).await
}

#[get("/averages_d/<event>")]
pub async fn averages_event(event: String, db: &State<DatabaseConnection>) -> String {
    averages_impl(Some(&event), db).await
}

async fn averages_impl(event: Option<&str>, db: &State<DatabaseConnection>) -> String {
    let avgfunc = YEARSAVG[&SETTINGS.year];
    
    let e = match avgfunc(event.map(|x| x.to_string()), db).await {
        Ok(a) => a,
        Err(a) => {
            return format!("Error! Could not find average: {a}");
        },
    };
    e.to_string()
}
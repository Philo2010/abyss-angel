use std::error::Error;

use rocket::State;
use rocket::{post, serde::json::Json};
use sea_orm::{DatabaseConnection, EntityOrSelect, EntityTrait};
use sea_orm::dynamic::DynSelector;
use serde_json::Value;

use crate::{SETTINGS, models, sexymac};
use crate::user::{YEARSAVG, YEARSINSERT};


#[get("/averages_d")]
pub async fn averages_empty(db: &State<DatabaseConnection>) -> String {
    let event = sexymac::get_event_default(db.inner()).await;

    averages_impl(event, db).await
}

#[get("/averages_d/<event>")]
pub async fn averages_event(event: String, db: &State<DatabaseConnection>) -> String {
    averages_impl(Some(event), db).await
}

async fn averages_impl(event: Option<String>, db: &State<DatabaseConnection>) -> String {
    let avgfunc = YEARSAVG[&SETTINGS.year];
    
    let e = match avgfunc(event, db).await {
        Ok(a) => a,
        Err(a) => {
            return format!("Error! Could not find average: {a}");
        },
    };
    e.to_string()
}
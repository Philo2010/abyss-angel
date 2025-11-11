use std::error::Error;

use rocket::State;
use rocket::form::Form;
use rocket::{post, serde::json::Json};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::SETTINGS;
use crate::user::{YEARSAVG, YEARSINSERT, YEARSSEARCH};


#[derive(FromForm)]
struct SearchForm {
    event: Option<String>,
    scouter: Option<String>,
    team: Option<i32>
}


#[post("/search", data="<body>")]
async fn search(body: Form<SearchForm>, db: &State<DatabaseConnection>) -> String {
    let avgfunc = YEARSSEARCH[&SETTINGS.year];

    let e = match avgfunc(body.event.clone(), body.scouter.clone(), body.team, db).await {
        Ok(a) => a,
        Err(a) => {
            let errormessage = format!("Error! Could not find avgrage: {a}");
            return errormessage;
        },
    };


    e.to_string()
}

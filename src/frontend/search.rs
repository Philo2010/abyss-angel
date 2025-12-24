
use rocket::State;
use rocket::form::Form;
use rocket::post;
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::{SETTINGS, sexymac};
use crate::user::YEARSSEARCH;


#[derive(FromForm)]
pub struct SearchForm {
    event: Option<String>,
    scouter: Option<String>,
    team: Option<i32>
}

#[derive(FromForm)]
pub struct SearchDefault {
    scouter: Option<String>,
    team: Option<i32>
}


#[post("/search_all", data="<body>")]
pub async fn search(body: Form<SearchForm>, db: &State<DatabaseConnection>) -> String {
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

#[post("/search", data="<body>")]
pub async fn search_default(body: Form<SearchDefault>, db: &State<DatabaseConnection>) -> Template {
    let avgfunc = YEARSSEARCH[&SETTINGS.year];

    let e = match avgfunc(sexymac::get_event_default(db.inner()).await, body.scouter.clone(), body.team, db).await {
        Ok(a) => a,
        Err(a) => {
            let errormessage = format!("Error! Could not find avgrage: {a}");
            return Template::render("error", context! {error: errormessage});
        },
    };


    Template::render("table", context! {entries: e})
}
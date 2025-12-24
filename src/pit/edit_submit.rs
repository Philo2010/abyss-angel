use rocket::{State, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::{SETTINGS, pit::pit2025::PITYEARSEDIT};




#[post("/pit/submit_edit", data="<data>")]
pub async fn edit_submit(data: Json<Value>, db: &State<DatabaseConnection>) -> Template {

    let editfunc = PITYEARSEDIT[&SETTINGS.year];

    match editfunc(db.inner(), &data).await {
        Ok(_) => {
            Template::render("suc", context! {message: "Edited data!"})
        },
        Err(a) => {
            Template::render("error", context! {error: format!("Problem editing data: {}", a.as_ref())})
        },
    }
}
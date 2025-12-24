use rocket::{State, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::{DatabaseConnection};

use crate::{SETTINGS, pit::pit2025};




#[post("/pit/insert", data = "<data>")]
pub async fn insert_pit(data: Json<serde_json::Value>, db: &State<DatabaseConnection>) -> Template {
    let insertfunc = pit2025::PITYEARSINSERT[&SETTINGS.year];

    match insertfunc(db.inner(), &data).await {
        Ok(_) => {
            Template::render("suc", context! {message: "Able to pit scout!"})
        },
        Err(a) => {
            Template::render("error", context! {error: format!("Failed to insert pit: {}", a.as_ref())})
        },
    }
}
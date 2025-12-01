use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;
use rocket::State;

use crate::{SETTINGS, pit::pit2025::PITYEARDELETE};


#[get("/pit/delete/<id>")]
pub async fn pit_delete(id: i32, db: &State<DatabaseConnection>) -> Template {
    let deletefunc = PITYEARDELETE[&SETTINGS.year];

    match deletefunc(db.inner(), id).await {
        Ok(_) => {
            Template::render("suc", context! {message: "Pit scout deleted!"})
        },
        Err(a) => {
            Template::render("error", context! {error: format!("Delete failed: {}", a.as_ref())})
        },
    }
}

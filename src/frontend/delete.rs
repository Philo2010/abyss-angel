use rocket::State;
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::{SETTINGS, user::YEARDELETE};



#[get("/delete/<id>")]
pub async fn delete_scout(id: i32, db: &State<DatabaseConnection>) -> Template {
    let deletfunc = YEARDELETE[&SETTINGS.year];

    match deletfunc(db.inner(), id).await {
        Ok(()) => {
            Template::render("suc", context! {message: "Deleted!"})
        }
        Err(a) => {
            Template::render("error", context! {error: format! {"Failed to delete: {}", a.as_ref()}})
        }
    }
}
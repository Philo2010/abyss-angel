use rocket::State;
use rocket_dyn_templates::{Template, context};
use sea_orm::DatabaseConnection;

use crate::{SETTINGS, user::YEARSGET};



#[get("/edit/<id>")]
pub async fn edit_page(id: i32, db: &State<DatabaseConnection>) -> Template {
    let getfunc = YEARSGET[&SETTINGS.year];

    let res = match getfunc(db.inner(), id).await {
        Ok(a) => a,
        Err(a) => {
            return Template::render("error", context! {error: format!("Problem getting data for the edit: {}", a.as_ref())});
        },
    };

    

    Template::render("edit", context! {entry: res})
}
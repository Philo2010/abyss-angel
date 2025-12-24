use rocket::State;
use rocket_dyn_templates::{Template, context};
use sea_orm::{DatabaseConnection};
use crate::{SETTINGS, pit::pit2025::PITYEARSGET};


#[get("/pit/edit_page/<id>")]
pub async fn edit_page(db: &State<DatabaseConnection>, id: i32) -> Template {
    let getfunc = PITYEARSGET[&SETTINGS.year];

    let model = match getfunc(db.inner(), id).await {
        Ok(a) => a,
        Err(a) => {
            return Template::render("error", context! {error: format!("Error getting pit data: {}", a.as_ref())});
        },
    };

    Template::render("pit_edit", context! { data: model })
}
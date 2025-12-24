use rocket::{State, form::Form};
use rocket_dyn_templates::Template;
use sea_orm::{DatabaseConnection, EntityTrait};
use rocket_dyn_templates::context;

use crate::setting;



#[derive(FromForm)]
pub struct SetEvent {
    event: String
}

#[post("/set_event", data = "<body>")]
pub async fn set_event(body: Form<SetEvent>, db: &State<DatabaseConnection>) -> Template {

    let setting: setting::dyn_settings::ActiveModel = match setting::dyn_settings::Entity::find().one(db.inner()).await {
        Ok(Some(a)) => {
            let mut e: setting::dyn_settings::ActiveModel = a.into();

            e.event = sea_orm::Set(body.event.clone());

            e
        },
        Ok(None) => {
            setting::dyn_settings::ActiveModel {
                event: sea_orm::Set(body.event.clone()),
                ..Default::default()
            }
        },
        Err(a) => {
            return Template::render("error", context! {error: format!("Database error: {a}")});
        },
    };

    

    match setting::dyn_settings::Entity::insert(setting).exec(db.inner()).await {
        Ok(_) => {
            Template::render("suc", context!{message: "Done!"})
        },
        Err(a) => {
            Template::render("error", context!{message: format!("Database error!: {a}")})
        },
    }
}
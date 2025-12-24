use rocket::{State, form::Form};
use rocket_dyn_templates::{Template, context};
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use uuid::Uuid;

use crate::{SETTINGS, auth::admin};

#[derive(FromForm)]
pub struct CreateUserForm {
    username: String,
    password: String
}

#[post("/create_user", data="<data>")]
pub async fn create_user(data: Form<CreateUserForm>, db: &State<DatabaseConnection>) -> Template {
    let hash = match bcrypt::hash(data.password.clone(), SETTINGS.bcrypt) {
        Ok(a) => a,
        Err(a) => {
            return Template::render("error", context! {error: format!("Could not gen Bcrypt: {a}")});
        },
    };

    let acvmodel = admin::ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        name: sea_orm::Set(data.username.clone()),
        bcrypt_hash: sea_orm::Set(hash),
    };

    match admin::ActiveModel::insert(acvmodel, db.inner()).await {
        Ok(_) => {
            Template::render("suc", context! {message: "User Created!"})
        },
        Err(a) => {
            Template::render("error", context! {error: format!("Could not insert into database: {a}")})
        },
    }
}
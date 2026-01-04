use rocket::{State, form::Form};
use rocket_dyn_templates::{Template, context};
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use uuid::Uuid;
use crate::SETTINGS;
use crate::entity::users;

#[derive(FromForm)]
pub struct CreateUserForm {
    username: String,
    password: String,
    is_admin: String, // should be a ("yes", "no") value
}

fn parse_out_string_bool<'a>(value: &'a str) -> bool {
    if value == "yes" {
        true
    } else {
        false
    }
}

#[post("/create_user", data="<data>")]
pub async fn create_user(data: Form<CreateUserForm>, db: &State<DatabaseConnection>) -> Template {
    let hash = match bcrypt::hash(data.password.clone(), SETTINGS.bcrypt) {
        Ok(a) => a,
        Err(a) => {
            return Template::render("error", context! {error: format!("Could not gen Bcrypt: {a}")});
        },
    };

    let is_admin = parse_out_string_bool(&data.is_admin);

    let acvmodel = users::ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        name: sea_orm::Set(data.username.clone()),
        is_admin: sea_orm::Set(is_admin),
        amount_of_warning: sea_orm::Set(0), //for now, you have no sins.... ;>
        bcrypt_hash: sea_orm::Set(hash),
    };

    match users::ActiveModel::insert(acvmodel, db.inner()).await {
        Ok(_) => {
            Template::render("suc", context! {message: "User Created!"})
        },
        Err(a) => {
            Template::render("error", context! {error: format!("Could not insert into database: {a}")})
        },
    }
}
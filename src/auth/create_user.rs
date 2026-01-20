use rocket::http::CookieJar;
use rocket::{State, form::Form};
use rocket_dyn_templates::{Template, context};
use rocket_okapi::okapi;
use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;
use crate::SETTINGS;
use crate::auth::check;
use crate::entity::users;
use crate::frontend::ApiResult;
use rocket_dyn_templates::serde::json::Json;

#[derive(FromForm, Deserialize, JsonSchema)]
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

//for backend init only (its not mounted at /api/)
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

//for backend init only (its not mounted at /api/)
#[rocket_okapi::openapi]
#[post("/api/create_user", data="<data>")]
pub async fn create_user_front(data: Json<CreateUserForm>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    if (!check::check(cookies, db).await) {

    }

    let hash = match bcrypt::hash(data.password.clone(), SETTINGS.bcrypt) {
        Ok(a) => a,
        Err(a) => {
            return Json(ApiResult::Error("Failed handled".to_string()));
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
            return Json(ApiResult::Success("OK".to_string()));
        },
        Err(a) => {
            return Json(ApiResult::Error("Failed handled".to_string()));
        },
    }
}
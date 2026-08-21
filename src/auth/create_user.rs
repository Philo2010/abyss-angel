use rocket::http::CookieJar;
use rocket::{State, form::Form};
use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::Deserialize;
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
    is_pick: String, // same ese and no
}

fn parse_out_string_bool(value: &str) -> bool {
    value == "yes"
}

//for backend init only (its not mounted at /api/)
#[post("/create_user", data="<data>")]
pub async fn create_user(data: Form<CreateUserForm>, db: &State<DatabaseConnection>) -> &'static str {
    let hash = match bcrypt::hash(data.password.clone(), SETTINGS.bcrypt) {
        Ok(a) => a,
        Err(_) => {
            return "Could not gen Bcrypt";
        },
    };

    let is_admin = parse_out_string_bool(&data.is_admin);
    let is_pick = parse_out_string_bool(&data.is_pick);

    let acvmodel = users::ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        name: sea_orm::Set(data.username.clone()),
        is_admin: sea_orm::Set(is_admin),
        amount_of_warning: sea_orm::Set(0), //for now, you have no sins.... ;>
        bcrypt_hash: sea_orm::Set(hash),
        is_pick: sea_orm::Set(is_pick),
    };

    match users::ActiveModel::insert(acvmodel, db.inner()).await {
        Ok(_) => {
            return "User Created!";
        },
        Err(_) => {
            return "Could not insert into database";
        },
    }
}

//for backend init only (its not mounted at /api/)
#[rocket_okapi::openapi]
#[post("/api/create_user", data="<data>")]
pub async fn create_user_front(data: Json<CreateUserForm>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    if !check::check(cookies, db).await {
        return Json(ApiResult::Error("Not Admin".to_string()));
    }


    let hash = match bcrypt::hash(data.password.clone(), SETTINGS.bcrypt) {
        Ok(a) => a,
        Err(_) => {
            return Json(ApiResult::Error("Failed handled".to_string()));
        },
    };

    let is_admin = parse_out_string_bool(&data.is_admin);
    let is_pick = parse_out_string_bool(&data.is_pick);


    let acvmodel = users::ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        name: sea_orm::Set(data.username.clone()),
        is_admin: sea_orm::Set(is_admin),
        amount_of_warning: sea_orm::Set(0), //for now, you have no sins.... ;>
        bcrypt_hash: sea_orm::Set(hash),
        is_pick: sea_orm::Set(is_pick)
    };

    match users::ActiveModel::insert(acvmodel, db.inner()).await {
        Ok(_) => {
            Json(ApiResult::Success("OK".to_string()))
        },
        Err(_) => {
            Json(ApiResult::Error("Failed handled".to_string()))
        },
    }
}
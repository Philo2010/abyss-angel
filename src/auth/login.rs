use rocket::serde::json::Json;
use rocket::{State, form::Form};
use rocket_dyn_templates::{Template, context};
use schemars::JsonSchema;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use rocket::http::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::UUID_COOKIE_NAME;
use crate::entity::users;



#[derive(Deserialize, JsonSchema)]
pub struct LoginForm {
    username: String,
    password: String
}

#[derive(Responder, Serialize, JsonSchema)]
pub enum LoginRes {
    Success(String),
    Error(String),
}

#[rocket_okapi::openapi]
#[post("/api/login", data="<data>")]
pub async fn login(data: Json<LoginForm>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<LoginRes> {

    println!("{}", &data.username);
    let a = match users::Entity::find()
        .filter(users::Column::Name.eq(&data.username)).one(db.inner()).await {
            Ok(Some(a)) => a,
            Ok(None) => {
                return Json(LoginRes::Error("No user found!".to_string()));
            },
            Err(a) => {
                return Json(LoginRes::Error(format!("Error when trying to find user: {a}")));
            },
        };
    
    let res = match bcrypt::verify(&data.password, &a.bcrypt_hash) {
        Ok(a) => a,
        Err(a) => {
            return Json(LoginRes::Error(format!("Could not check hash!: {a}")));
        },
    };


    if res {
        //Good
        cookies.add(Cookie::new(UUID_COOKIE_NAME, a.id.to_string()));
        Json(LoginRes::Success("Your logined in!".to_string()))
        
    } else {
        Json(LoginRes::Error("Wrong password!".to_string()))
    }
}
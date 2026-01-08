use rocket::serde::json::Json;
use rocket::{State, form::Form};
use rocket_dyn_templates::{Template, context};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use rocket::http::{Cookie, CookieJar};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::UUID_COOKIE_NAME;
use crate::entity::users;



#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String
}

#[derive(Responder)]
pub enum LoginRes {
    #[response(status = 200)]
    Success(Json<String>),
    #[response(status = 400)]
    Error(Json<String>),
}


#[post("/api/login", data="<data>")]
pub async fn login(data: Json<LoginForm>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> LoginRes {

    let a = match users::Entity::find()
        .filter(users::Column::Name.eq(&data.username)).one(db.inner()).await {
            Ok(Some(a)) => a,
            Ok(None) => {
                return LoginRes::Error(Json("No user found!".to_string()));
            },
            Err(a) => {
                return LoginRes::Error(Json(format!("Error when trying to find user: {a}")));
            },
        };
    
    let res = match bcrypt::verify(&data.password, &a.bcrypt_hash) {
        Ok(a) => a,
        Err(a) => {
            return LoginRes::Error(Json(format!("Could not check hash!: {a}")));
        },
    };


    if res {
        //Good
        cookies.add(Cookie::new(UUID_COOKIE_NAME, a.id.to_string()));
        LoginRes::Success(Json("Your logined in!".to_string()))
        
    } else {
        LoginRes::Error(Json("Wrong password!".to_string()))
    }
}
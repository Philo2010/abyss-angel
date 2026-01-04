use rocket::{State, form::Form};
use rocket_dyn_templates::{Template, context};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use rocket::http::{Cookie, CookieJar};

use crate::auth::UUID_COOKIE_NAME;
use crate::entity::users;



#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String
}


#[post("/login_user", data="<data>")]
pub async fn login(data: Form<LoginForm>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Template {

    let a = match users::Entity::find()
        .filter(users::Column::Name.eq(&data.username)).one(db.inner()).await {
            Ok(Some(a)) => a,
            Ok(None) => {
                return Template::render("error", context! {error: "No user found!"});
            },
            Err(a) => {
                return Template::render("error", context! {error: format!("Error when trying to find user: {a}")});
            },
        };
    
    let res = match bcrypt::verify(&data.password, &a.bcrypt_hash) {
        Ok(a) => a,
        Err(a) => {
            return Template::render("error", context! {error: format!("Could not check hash!: {a}")});
        },
    };


    if res {
        //Good
        cookies.add(Cookie::new(UUID_COOKIE_NAME, a.id.to_string()));
        Template::render("suc", context! {message: "Logined!"})
    } else {
        Template::render("error", context! {error: format!("Wrong password!")})
    }
}
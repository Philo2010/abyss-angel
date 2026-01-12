use rocket::{State, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{auth::{self, get_by_user::get_by_username}, entity::users, frontend::ApiResult};


#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ResetPasswordForm {
    pub user: String,
    pub new_password: String,
}

#[rocket_okapi::openapi]
#[post("/api/misc/reset_password", data="<data>")]
pub async fn reset_password(db: &State<DatabaseConnection>, data: Json<ResetPasswordForm>) -> Json<ApiResult<String>> {
    let data_of_form = data.into_inner();
    let uuid = match get_by_username(&data_of_form.user, db).await {
        Ok(a) => a,
        Err(a) => {
            match a {
                crate::auth::get_by_user::AuthGetUuidError::UserIsNotHere => {
                    return Json(ApiResult::Error("Could not find user!".to_string()));
                },
                crate::auth::get_by_user::AuthGetUuidError::DatabaseError(db_err) => {
                    return Json(ApiResult::Error(format!("Database error: {db_err}")));
                },
            }
        },
    };
    match auth::reset_password::reset_password(&uuid, &data_of_form.new_password, db).await {
        Ok(a) => {
            Json(ApiResult::Success("Done!".to_string()))
        },
        Err(a) => {
            Json(ApiResult::Error(format!("Error: {a}")))
        },
    }
}
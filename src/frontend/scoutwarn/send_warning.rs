use rocket::{State, http::CookieJar, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{auth::{self, get_by_cookie, get_by_user::{AuthGetUuidError, get_by_username}}, frontend::{ApiResult, scoutwarn}, scoutwarn::get_warning::ReturnWarning};



#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Message {
    //sender will be gotten by cookies
    resv: String,
    message: String,
}

#[rocket_okapi::openapi]
#[post("/api/scoutwarn/send", data="<data>")]
pub async fn send_scoutwarn(data: Json<Message>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    let resv = match get_by_username(&data.resv, db).await {
        Ok(a) => a,
        Err(a) => {
            match a {
                AuthGetUuidError::UserIsNotHere => {
                            return Json(ApiResult::Error("Could not find user!".to_string()));
                        },
                AuthGetUuidError::DatabaseError(db_err) => {
                    return Json(ApiResult::Error(format!("Failed to get username: {db_err}")));
                },
            }
        },
    };
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    let uuid = match get_by_cookie::get(cookies) {
        Some(a) => a,
        None => {
            return Json(ApiResult::Error("Not logined in!".to_string()));
        },
    };

    let warning = crate::scoutwarn::send_warning::SendWarning {
        sender: Some(uuid),
        receiver: resv,
        message: data.message.clone()
    };

    match crate::scoutwarn::send_warning::send_warning(warning, db.inner()).await {
        Ok(_) => {
            return Json(ApiResult::Success("Able to send message, they shall pay for there sins".to_string()));
        },
        Err(a) => {
            return Json(ApiResult::Error(format!("Database error while sending scouter: {a}")));
        },
    };
}
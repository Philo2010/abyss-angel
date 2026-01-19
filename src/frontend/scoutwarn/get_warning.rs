use rocket::{State, http::CookieJar, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{auth::{get_by_cookie, get_by_user::get_by_uuid}, frontend::{ApiResult, scoutwarn}, scoutwarn::get_warning::ReturnWarning};


#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ReturnWarningSend {
    pub unsent_id: i32,
    pub id: i32,
    pub sender: String,
    pub receiver: String,
    pub message: String,
}

#[rocket_okapi::openapi]
#[get("/api/scoutwarn/getall")]
pub async fn get_scoutwarn(db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<Vec<ReturnWarningSend>>> {
    let uuid = match get_by_cookie::get(cookies) {
        Some(a) => a,
        None => {
            return Json(ApiResult::Error("Not logined in!".to_string()));
        },
    };

    match crate::scoutwarn::get_warning::get_warning(uuid, db).await {
        Ok(a) => {
            let mut send: Vec<ReturnWarningSend> = Vec::new();
            for warn in a {
                let sender;
                if warn.message.sender.is_none() {
                    sender = "Unkwown".to_string();
                } else {
                    sender = get_by_uuid(&warn.message.sender.unwrap(), db).await.unwrap();
                }
                let rec = get_by_uuid(&warn.message.receiver, db).await.unwrap();
                let thing = ReturnWarningSend {
                    unsent_id: warn.unsent_id,
                    id: warn.message.id,
                    sender: sender,
                    receiver: rec,
                    message: warn.message.message};
                send.push(thing);
            }

            return Json(ApiResult::Success(send));
        },
        Err(a) => {
            return Json(ApiResult::Error(format!("Database error while trying to get games: {a}")));
        },
    }
}
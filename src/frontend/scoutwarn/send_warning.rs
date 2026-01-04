use rocket::{State, http::CookieJar, serde::json::Json};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{auth::{self, get_by_cookie}, frontend::scoutwarn, scoutwarn::get_warning::ReturnWarning};


#[derive(Responder)]
pub enum ScoutWarnSendResult {
    #[response(status = 200)]
    Success(Json<String>),
    #[response(status = 400)]
    Error(Json<String>),
}


#[derive(Serialize, Deserialize)]
pub struct Message {
    //sender will be gotten by cookies
    resv: Uuid,
    message: String,
}


#[post("/api/scoutwarn/send", data="<data>")]
pub async fn send_scoutwarn(data: Json<Message>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> ScoutWarnSendResult {
    if !auth::check::check(cookies, db).await {
        return ScoutWarnSendResult::Error(Json("Need to be admin!".to_string()));
    }
    let uuid = match get_by_cookie::get(cookies) {
        Some(a) => a,
        None => {
            return ScoutWarnSendResult::Error(Json("Not logined in!".to_string()));
        },
    };

    let warning = crate::scoutwarn::send_warning::SendWarning {
        sender: Some(uuid),
        receiver: data.resv,
        message: data.message.clone()
    };

    match crate::scoutwarn::send_warning::send_warning(warning, db).await {
        Ok(_) => {
            return ScoutWarnSendResult::Success(Json("Able to send message, they shall pay for there sins".to_string()));
        },
        Err(a) => {
            return ScoutWarnSendResult::Error(Json(format!("Database error while sending scouter: {a}")));
        },
    };
}
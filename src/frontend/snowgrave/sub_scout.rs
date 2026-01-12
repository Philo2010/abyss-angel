


use std::fmt::format;

use rocket::State;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::post;
use rocket::serde::json::Json;
use rocket_dyn_templates::{Template, context};
use schemars::JsonSchema;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::auth::get_by_user::{AuthGetUuidError, get_by_username};
use crate::backenddb::game::{GamesGraph, graph_game};
use crate::frontend::ApiResult;
use crate::frontend::pit::get::get;
use crate::snowgrave::snowgrave_edit_scouter::EditSnow;
use crate::{SETTINGS, auth, sexymac};


#[derive(JsonSchema, Deserialize, Serialize)]
pub struct SubScoutForm {
    og: String,
    replacement: String
}


#[rocket_okapi::openapi]
#[post("/api/sub_scout", data = "<body>")]
pub async fn sub_scout(body: Json<SubScoutForm>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    let og_uuid = match get_by_username(&body.og, db).await {
        Ok(a) => a,
        Err(a) => {
            match a {
                    AuthGetUuidError::UserIsNotHere => {
                        return Json(ApiResult::Error("The og was not found".to_string()));
                    }
                    AuthGetUuidError::DatabaseError(db_err) => {
                        return Json(ApiResult::Error(format!("Db error while getting OG uuid: {db_err}")));
                    }
                }
        },
    };
    let replacement_uuid = match get_by_username(&body.replacement, db).await {
        Ok(a) => a,
        Err(a) => {
            match a {
                    AuthGetUuidError::UserIsNotHere => {
                        return Json(ApiResult::Error("The replacement was not found".to_string()));
                    }
                    AuthGetUuidError::DatabaseError(db_err) => {
                        return Json(ApiResult::Error(format!("Db error while getting OG uuid: {db_err}")));
                    }
                }
        },
    };

    match crate::snowgrave::sub_scout::sub_scout(db, &og_uuid, &replacement_uuid).await {
        Ok(a) => {
            Json(ApiResult::Success("Done!".to_string()))
        },
        Err(a) => {
            Json(ApiResult::Error(format!("Database Error while doing sub: {a}")))
        },
    }
}
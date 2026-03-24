



use rocket::{State, http::CookieJar, post, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{auth::{self, get_by_user::{AuthGetUuidError, get_by_username}}, frontend::ApiResult};



#[derive(JsonSchema, Deserialize, Serialize)]
pub struct SubScoutForm {
    og: String,
    replacement: String,
    /// When provided, only replace assignments for these game IDs (selective sub).
    /// When None or empty, replace ALL pending assignments (full sub).
    #[serde(default)]
    game_ids: Option<Vec<i32>>,
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

    match crate::snowgrave::sub_scout::sub_scout(db, &og_uuid, &replacement_uuid, body.game_ids.as_deref()).await {
        Ok(_) => {
            Json(ApiResult::Success("Done!".to_string()))
        },
        Err(a) => {
            Json(ApiResult::Error(format!("Database Error while doing sub: {a}")))
        },
    }
}
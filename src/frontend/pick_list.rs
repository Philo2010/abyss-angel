
use rocket::{State, http::CookieJar, serde::json::Json};
use sea_orm::DatabaseConnection;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{auth, frontend::ApiResult, pick_list};

#[derive(Deserialize, JsonSchema)]
pub struct GetPickListRequest {
    include_midway: bool,
}

#[rocket_okapi::openapi]
#[post("/api/pick_list", data = "<data>")]
pub async fn get_pick_list(data: Json<GetPickListRequest>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<pick_list::get::PickLists>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    let res = match pick_list::get::get(db, data.include_midway).await {
        Ok(a) => a,
        Err(a) => {
            return Json(ApiResult::Error(a.to_string()));
        }
    };

    Json(ApiResult::Success(res))
}

#[rocket_okapi::openapi]
#[post("/api/pick_list/set", data = "<data>")]
pub async fn set_pick_list(data: Json<pick_list::set::SetPick>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<String>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }

    match pick_list::set::set(data.into_inner(), db).await {
        Ok(()) => Json(ApiResult::Success("updated".to_string())),
        Err(a) => Json(ApiResult::Error(a.to_string())),
    }
}

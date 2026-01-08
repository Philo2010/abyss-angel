use rocket::{State, form::Form, serde::json::Json};
use rocket_dyn_templates::Template;
use schemars::JsonSchema;
use sea_orm::{DatabaseConnection, EntityTrait, entity};
use rocket_dyn_templates::context;
use serde::Deserialize;
use serde_json::Value;

use crate::{frontend::ApiResult, setting};

#[derive(Deserialize, JsonSchema)]
pub struct SetEvent {
    event: String
}

#[post("/api/set_event", data = "<body>")]
pub async fn set_event(body: Json<SetEvent>, db: &State<DatabaseConnection>) -> Json<ApiResult<String>> {

    let setting: crate::entity::dyn_settings::ActiveModel = match crate::entity::dyn_settings::Entity::find().one(db.inner()).await {
        Ok(Some(a)) => {
            let mut e: crate::entity::dyn_settings::ActiveModel = a.into();

            e.event = sea_orm::Set(body.event.clone());

            e
        },
        Ok(None) => {
            crate::entity::dyn_settings::ActiveModel {
                event: sea_orm::Set(body.event.clone()),
                ..Default::default()
            }
        },
        Err(a) => {
            return Json(ApiResult::Error(format!("Could not find settings mod: {a}")));
        },
    };

    

    match crate::entity::dyn_settings::Entity::insert(setting).exec(db.inner()).await {
        Ok(_) => {
            Json(ApiResult::Success("Done!".to_string()))
        },
        Err(a) => {
            Json(ApiResult::Error(format!("Failed to insert event: {a}")))
        },
    }
}

#[get("/api/get_event")]
pub async fn get_event(db: &State<DatabaseConnection>) -> Json<ApiResult<String>> {

    let setting = match crate::entity::dyn_settings::Entity::find().one(db.inner()).await {
        Ok(Some(a)) => {a},
        Ok(None) => {
            return Json(ApiResult::Error(format!("Could not find settings mod")));
        },
        Err(a) => {
            return Json(ApiResult::Error(format!("Could not find settings mod: {a}")));
        },
    };


    Json(ApiResult::Success(setting.event))
}
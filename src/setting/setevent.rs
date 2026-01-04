use rocket::{State, form::Form, serde::json::Json};
use rocket_dyn_templates::Template;
use sea_orm::{DatabaseConnection, EntityTrait, entity};
use rocket_dyn_templates::context;
use serde::Deserialize;
use serde_json::Value;

use crate::setting;

#[derive(Deserialize)]
pub struct SetEvent {
    event: String
}

#[derive(Responder)]
pub enum EventRespone {
    #[response(status = 200)]
    Success(Json<String>),
    #[response(status = 400)]
    Error(Json<String>),
}

#[post("/set_event", data = "<body>")]
pub async fn set_event(body: Json<SetEvent>, db: &State<DatabaseConnection>) -> EventRespone {

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
            return EventRespone::Error(Json(format!("Could not find settings mod: {a}")));
        },
    };

    

    match crate::entity::dyn_settings::Entity::insert(setting).exec(db.inner()).await {
        Ok(_) => {
            EventRespone::Success(Json("Done!".to_string()))
        },
        Err(a) => {
            EventRespone::Error(Json(format!("Failed to insert event: {a}")))
        },
    }
}

#[get("/get_event")]
pub async fn get_event(db: &State<DatabaseConnection>) -> EventRespone {

    let setting = match crate::entity::dyn_settings::Entity::find().one(db.inner()).await {
        Ok(Some(a)) => {a},
        Ok(None) => {
            return EventRespone::Error(Json(format!("Could not find settings mod")));
        },
        Err(a) => {
            return EventRespone::Error(Json(format!("Could not find settings mod: {a}")));
        },
    };


    EventRespone::Success(Json(setting.event))
}
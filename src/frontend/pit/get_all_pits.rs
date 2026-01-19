use rocket::{State, response::stream::Event, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use crate::{entity::pit_upcoming, frontend::ApiResult, setting::setevent::get_event_inner};


#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PitEvent {
    pub id: i32,
    pub team: i32,
    pub is_ab_team: bool,
    pub is_done: bool,
    pub event_code: String,
}


#[rocket_okapi::openapi]
#[get("/api/pit/get_all")]
pub async fn pit_get_all(db: &State<DatabaseConnection>) -> Json<ApiResult<Vec<PitEvent>>> {
    let cur_event = match get_event_inner(db).await {
        Ok(a) => a,
        Err(a) => {
            return Json(ApiResult::Error(format!("DB error while getting event: {a}")));
        },
    };

    let events: Vec<PitEvent> = match pit_upcoming::Entity::find()
        .filter(pit_upcoming::Column::EventCode.eq(cur_event))
        .all(db.inner()).await {
            Ok(a) => a,
            Err(a) => {
                return Json(ApiResult::Error(format!("Db error while getting events: {a}")));
            },
        }.into_iter().map(|x| {
            PitEvent {
                id: x.id,
                team: x.team,
                is_ab_team: x.is_ab_team,
                event_code: x.event_code,
                is_done: x.is_done
            }
        }).collect();
    


    Json(ApiResult::Success(events))
}
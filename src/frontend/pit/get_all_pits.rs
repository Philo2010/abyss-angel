use rocket::{State, serde::json::Json};
use schemars::JsonSchema;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use crate::{auth::get_by_user::get_by_uuid, entity::pit_upcoming, frontend::ApiResult, setting::setevent::get_event_inner};


#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PitEvent {
    pub id: i32,
    pub team: i32,
    pub is_ab_team: bool,
    pub is_done: bool,
    pub event_code: String,
    pub user: Option<String>,
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

    let rows = match pit_upcoming::Entity::find()
        .filter(pit_upcoming::Column::EventCode.eq(cur_event))
        .all(db.inner()).await {
            Ok(a) => a,
            Err(a) => {
                return Json(ApiResult::Error(format!("Db error while getting events: {a}")));
            },
        };

    let mut events: Vec<PitEvent> = Vec::with_capacity(rows.len());
    for x in rows {
        let username = match x.user {
            Some(uuid) => get_by_uuid(&uuid, db.inner()).await.ok(),
            None => None,
        };
        events.push(PitEvent {
            id: x.id,
            team: x.team,
            is_ab_team: x.is_ab_team,
            event_code: x.event_code,
            is_done: x.is_done,
            user: username,
        });
    }

    Json(ApiResult::Success(events))
}
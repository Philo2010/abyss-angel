use schemars::JsonSchema;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::Serialize;
use uuid::Uuid;

use crate::{entity::pit_upcoming};


#[derive(Serialize, JsonSchema)]
pub struct PitScouterInstance {
    id: i32,
    event: String,
    team: i32,
    is_ab_team: bool,
    is_sum: bool
}


pub async fn get_scouters_pit(scouter: Uuid, event: &String, db: &DatabaseConnection) -> Result<Vec<PitScouterInstance>, DbErr> {
    let upcoming_pit = pit_upcoming::Entity::find()
        .filter(pit_upcoming::Column::EventCode.eq(event))
        .filter(pit_upcoming::Column::User.eq(Some(scouter))).all(db).await?;

    let mut instance: Vec<PitScouterInstance> = Vec::with_capacity(upcoming_pit.len());
    for pit in upcoming_pit {
        let is_sum = match pit.pit_header_id {
            Some(_) => true,
            None => false,
        };
        instance.push(PitScouterInstance {
            id: pit.id ,
            event: pit.event_code,
            team: pit.team,
            is_ab_team: pit.is_ab_team,
            is_sum: is_sum
        });
    }

    Ok(instance)
}
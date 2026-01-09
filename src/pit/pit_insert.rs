use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait};
use serde::Deserialize;

use crate::{entity::pit_upcoming, pit::pit::{self, PitHeaderInsert, PitInsert, PitInsertsSpecific}};

#[derive(Deserialize, JsonSchema)]
pub struct PitInsertForm {
    pub pit_insert_id: i32,
    pub pit: PitInsertsSpecific,
}

pub async fn pit_insert(db: &DatabaseConnection, data: PitInsertForm) -> Result<(), DbErr> {
    let pit_data = match pit_upcoming::Entity::find_by_id(data.pit_insert_id).one(db).await? {
        Some(a) => a,
        None => {
            return Err(DbErr::Custom("Invaid ID!".to_string()));
        },
    };
    let user = match pit_data.user {
        Some(a) => a,
        None => {
            return Err(DbErr::Custom("This scouter has not been selected yet!".to_string()));
        },
    };
    let insert_pit = PitInsert {
        header: PitHeaderInsert {
            user: user,
            team: pit_data.team,
            is_ab_team: pit_data.is_ab_team,
            event_code: pit_data.event_code.clone(),
        },
        pit: data.pit,
    };
    let res = pit::pit_insert(insert_pit, db).await?;
    let mut pit_active: pit_upcoming::ActiveModel = pit_data.into();
    pit_active.pit_header_id = Set(Some(res));
    let _res2 = pit_active.update(db).await?;


    Ok(())
}
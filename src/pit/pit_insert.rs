use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::{entity::{pit_header, pit_upcoming}, pit::pit::{self, PitHeaderInsert, PitInsert, PitInsertsSpecific}};

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

    if pit_data.pit_header_id.is_some() || pit_data.is_done {
        return Err(DbErr::Custom("Pit has already been submitted. Use edit instead.".to_string()));
    }

    let existing_header = pit_header::Entity::find()
        .filter(pit_header::Column::Team.eq(pit_data.team))
        .filter(pit_header::Column::IsAbTeam.eq(pit_data.is_ab_team))
        .filter(pit_header::Column::EventCode.eq(pit_data.event_code.clone()))
        .one(db)
        .await?;

    if existing_header.is_some() {
        return Err(DbErr::Custom("Pit has already been submitted. Use edit instead.".to_string()));
    }

    let insert_pit = PitInsert {
        header: PitHeaderInsert {
            user,
            team: pit_data.team,
            is_ab_team: pit_data.is_ab_team,
            event_code: pit_data.event_code.clone(),
        },
        pit: data.pit,
    };
    let res = pit::pit_insert(insert_pit, db).await?;
    let mut pit_active: pit_upcoming::ActiveModel = pit_data.into();
    pit_active.pit_header_id = Set(Some(res));
    pit_active.is_done = Set(true);
    let _res2 = pit_active.update(db).await?;


    Ok(())
}
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::{entity::pit_upcoming, pit::pit::{self, PitEditSpecific}};

pub struct PitEditForm {
    pub pit_insert_id: i32,
    pub pit: PitEditSpecific,
}



pub async fn pit_edit(db: &DatabaseConnection, data: PitEditForm) -> Result<(), DbErr> {
    let pit_upcoming_data = match pit_upcoming::Entity::find_by_id(data.pit_insert_id).one(db).await? {
        Some(a) => a,
        None => {
            return Err(DbErr::Custom("pit_insert_id is invaild!".to_string()));
        },
    };

    let header_id = match pit_upcoming_data.pit_header_id {
        Some(a) => a,
        None => {
            return Err(DbErr::Custom("Pit has not even been submited yet!".to_string()));
        },
    };

    let _res = pit::pit_edit(data.pit, db, header_id).await?;

    todo!()
}
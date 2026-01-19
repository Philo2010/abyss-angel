use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait};
use uuid::Uuid;
use sea_orm::Set;

use crate::entity::pit_upcoming;

#[derive(Clone)]
pub struct AssignScoutForm {
    pub scouts: Vec<Uuid>,
    //first you is the index, next is the id to pit_upcoming
    pub asignment: Vec<(usize, i32)>
}

pub async fn assign_pit_scouts(db: &DatabaseConnection, data: AssignScoutForm) -> Result<(), DbErr> {
    for assment in data.asignment {
        let uuid = match data.scouts.get(assment.0) {
            Some(a) => *a,
            None => { 
                return Err(DbErr::Custom("There is a scouter out of index!".to_string()));
            },
        };
        let pit_upcoming = match pit_upcoming::Entity::find_by_id(assment.1).one(db).await? {
            Some(a) => a,
            None => {
                return Err(DbErr::Custom(format!("Pit Upcoming Id is invaild!: {}", assment.1)));
            },
        };
        let mut pit_active: pit_upcoming::ActiveModel = pit_upcoming.into();
        pit_active.user = Set(Some(uuid));
        pit_active.update(db).await?;
    }
    Ok(())
}
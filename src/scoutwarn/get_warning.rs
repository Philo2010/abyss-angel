use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, DbErr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity::{unsent_warning, warning};

#[derive(Serialize, Deserialize)]
pub struct ReturnWarning {
    pub unsent_id: i32,
    pub message: warning::Model,
}


pub async fn get_warning(scout_id: Uuid, db: &DatabaseConnection) -> Result<Vec<ReturnWarning>, DbErr> {
    let warnings_id = unsent_warning::Entity::find()
        .filter(unsent_warning::Column::Receiver.eq(scout_id)).all(db).await?;

    let warning = warning::Entity::find()
        .filter(warning::Column::Id.is_in(warnings_id.iter().map(|x| x.message_id).collect::<Vec<i32>>())).all(db).await?; 

   use std::collections::HashMap;

    let warning_map: HashMap<i32, warning::Model> = warning
        .into_iter()
        .map(|w| (w.id, w))
        .collect();

    let final_warnings: Vec<ReturnWarning> = warnings_id
        .into_iter()
        .filter_map(|uw| {
            warning_map.get(&uw.message_id).map(|w| ReturnWarning{unsent_id: uw.id, message: w.clone()})
        })
        .collect();


    Ok(final_warnings)

}
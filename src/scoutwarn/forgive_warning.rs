use sea_orm::{DatabaseConnection, DbErr, EntityTrait};
use uuid::Uuid;

use crate::entity::unsent_warning;




//we forgive, but dont forget, so we dont delete the real message and do not reduce the count, here, there is no christ to save you from your sins HAHAHAHHAHAHAHHAHAH
pub async fn forgive_warning(warnings_id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    let res = unsent_warning::Entity::delete_by_id(warnings_id).exec(db).await?;

    Ok(())
}
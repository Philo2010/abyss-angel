use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, DatabaseConnection, DbErr, EntityTrait};
use uuid::Uuid;

use crate::entity::{unsent_warning, users, warning};



pub struct SendWarning {
    pub sender: Option<Uuid>, //None is snowgrave
    pub receiver: Uuid,
    pub message: String,
}

pub async fn send_warning(message: SendWarning, db: &DatabaseConnection) -> Result<(), DbErr> {
    let active_message = warning::ActiveModel {
        id: NotSet,
        sender: Set(message.sender),
        receiver: Set(message.receiver),
        message: Set(message.message),
    };

    let id = warning::Entity::insert(active_message).exec(db).await?.last_insert_id;

    let active_unset = unsent_warning::ActiveModel {
        id: NotSet,
        receiver: Set(message.receiver),
        message_id: Set(id),
    };
    unsent_warning::Entity::insert(active_unset).exec(db).await?;

    //add the warning to the count
    let user = users::Entity::find_by_id(message.receiver).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find the user!".to_string()))?;
    let mut user_active: users::ActiveModel = user.into();
    user_active.amount_of_warning = Set(user_active.amount_of_warning.unwrap() + 1);


    Ok(())
}
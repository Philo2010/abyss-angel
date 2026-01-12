use bcrypt::bcrypt;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{SETTINGS, entity::users};






pub async fn reset_password(user: &Uuid, new_password: &String, db: &DatabaseConnection) -> Result<(), DbErr> {
    let mut user_data: users::ActiveModel = users::Entity::find_by_id(*user).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find user!".to_string()))?.into();

    let pass_hash = match bcrypt::hash(new_password, SETTINGS.bcrypt) {
        Ok(a) => a,
        Err(a) => {
            return Err(DbErr::Custom(format!("Failed to create hash of password: {a}")));
        },
    };
    user_data.bcrypt_hash = Set(pass_hash);
    match user_data.update(db).await {
        Ok(a) => {
            Ok(())
        },
        Err(a) => {
            Err(a)
        },
    }
}
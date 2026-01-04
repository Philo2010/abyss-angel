use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect};
use uuid::Uuid;
use crate::entity::users;


pub enum AuthGetUuidError {
    UserIsNotHere,
    DatabaseError(DbErr)
}

pub async fn get_by_username<'a>(username: &'a str, db: &DatabaseConnection) -> Result<Uuid, AuthGetUuidError> {
 
    match users::Entity::find()
        .filter(users::Column::Name.eq(username))
        .one(db)
        .await {
            Err(a) => {
                Err(AuthGetUuidError::DatabaseError(a))
            },
            Ok(Some(a)) => {
                Ok(a.id)
            },
            Ok(None) => {
                Err(AuthGetUuidError::UserIsNotHere)
            }
    }
}

pub async fn get_by_uuid<'a>(username: &Uuid, db: &DatabaseConnection) -> Result<String, AuthGetUuidError> {
    match users::Entity::find()
        .filter(users::Column::Id.eq(*username))
        .one(db)
        .await {
            Err(a) => {
                Err(AuthGetUuidError::DatabaseError(a))
            },
            Ok(Some(a)) => {
                Ok(a.name)
            },
            Ok(None) => {
                Err(AuthGetUuidError::UserIsNotHere)
            }
    }
}
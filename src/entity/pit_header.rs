use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pit_header")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user: Uuid,
    #[sea_orm(unique_key = "item")]
    pub team: i32,
    #[sea_orm(unique_key = "item")]
    pub is_ab_team: bool,
    #[sea_orm(unique_key = "item")]
    pub event_code: String,
    pub created_at: DateTime,
    pub pit_data: i32,
    pub pit_type: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
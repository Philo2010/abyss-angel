use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pit_upcoming")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user: Option<Uuid>,
    pub team: i32,
    pub is_ab_team: bool,
    pub is_done: bool,

    pub event_code: String,
    pub pit_header_id: Option<i32>
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
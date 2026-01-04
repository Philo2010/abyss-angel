use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "warning")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub receiver: Uuid,
    pub message_id: i32,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::warning::Entity",
              from = "Column::MessageId"
              to = "super::warning::Column::Id")]
    Message
}

impl Related<super::warning::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Message.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
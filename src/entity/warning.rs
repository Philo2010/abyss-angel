use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Deserialize, Serialize)]
#[sea_orm(table_name = "warning")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub sender: Option<Uuid>, //If null then -> is snowgrave
    pub receiver: Uuid,
    pub message: String,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::unsent_warning::Entity")]
    Unsent
}

impl Related<super::unsent_warning::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Unsent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
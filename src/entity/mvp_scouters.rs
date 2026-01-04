use uuid::Uuid;
use sea_orm::entity::prelude::*;


use crate::entity::mvp_data;



#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "game_scouts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub scouter: Uuid,

    //Data for MVP -> Will be found when MVP scouts
    #[sea_orm(has_one)]
    pub data: Option<i32>
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::mvp_data::Entity")]
    MvpData,
    #[sea_orm(
        belongs_to = "super::upcoming_game::Entity",
        from = "Column::Id",
        to = "super::upcoming_game::Column::MvpId"
    )]
    Game
}

impl Related<super::mvp_data::Entity> for Relation {
    fn to() -> RelationDef {
        Self::MvpData.def()
    }
}

impl Related<super::upcoming_game::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Game.def()
    }
}


impl ActiveModelBehavior for ActiveModel {}
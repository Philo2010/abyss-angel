use sea_orm::entity::prelude::*;

use crate::entity::{game_scouts, sea_orm_active_enums::Stations, upcoming_game, upcoming_team};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "game_scouts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub game_id: i32,
    pub team_id: i32,

    pub scouter_id: Uuid,
    pub station: Stations, //Not needed but very useful

    pub done: bool,
    pub is_redo: bool,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Game,
    Team
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Game => Entity::belongs_to(upcoming_game::Entity)
                .from(Column::GameId)
                .to(upcoming_game::Column::Id)
                .into(),
            Self::Team => Entity::belongs_to(upcoming_team::Entity)
                .from(Column::TeamId)
                .to(upcoming_team::Column::Id)
                .into()
        }
    }
}

impl Related<upcoming_game::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Game.def()
    }
}

impl Related<upcoming_team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

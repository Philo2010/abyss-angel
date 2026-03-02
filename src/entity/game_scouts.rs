use sea_orm::entity::prelude::*;

use crate::entity::{
    scout_game_midway_insert,
    sea_orm_active_enums::Stations,
    upcoming_game,
    upcoming_team,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "game_scouts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub game_id: i32,
    pub team_id: i32,

    pub scouter_id: Uuid,
    pub station: Stations,
    pub game_midway: Option<i32>,

    pub done: bool,
    pub is_redo: bool,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Game,
    Team,
    GameData,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            // game_scouts belongs_to upcoming_game
            Self::Game => Entity::belongs_to(upcoming_game::Entity)
                .from(Column::GameId)
                .to(upcoming_game::Column::Id)
                .into(),

            // game_scouts belongs_to upcoming_team
            Self::Team => Entity::belongs_to(upcoming_team::Entity)
                .from(Column::TeamId)
                .to(upcoming_team::Column::Id)
                .into(),

            // game_scouts has_one scout_game_midway_insert
            Self::GameData => Entity::has_one(scout_game_midway_insert::Entity)
                .from(Column::GameMidway)
                .to(scout_game_midway_insert::Column::Id)
                .into(),
        }
    }
}

// For .find_also_related(upcoming_game::Entity)
impl Related<upcoming_game::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Game.def()
    }
}

// For .find_also_related(upcoming_team::Entity)
impl Related<upcoming_team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

// For .find_also_related(scout_game_midway_insert::Entity)
impl Related<scout_game_midway_insert::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GameData.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
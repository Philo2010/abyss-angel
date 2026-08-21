use sea_orm::entity::prelude::*;
use uuid::Uuid;

use crate::entity::{
    game_scouts,
    sea_orm_active_enums::{TournamentLevels, Stations},
    types::DefenceTarget,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mid_header")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user: Uuid,
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    pub total_score: i32,
    pub teleop_score: i32,
    pub auto_score: i32,
    pub defence: i32,
    pub comment: String,
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    pub created_at: DateTimeLocal,
    pub game_type_id: i32,
    pub game_id: i32,
    pub defence_main: bool,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub defence_target: Option<DefenceTarget>,
    pub auto_time: f32,
    pub dead: bool,
    pub dnf: bool,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    GameScout,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::GameScout => Entity::has_one(game_scouts::Entity)
                .from(Column::Id)
                .to(game_scouts::Column::GameMidway)
                .into(),
        }
    }
}

impl Related<game_scouts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GameScout.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
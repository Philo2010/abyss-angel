// =========================
// Domain + SeaORM Boundary
// =========================

use std::collections::HashMap;
use rocket::data::N;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::{
    entity::{
        game_scouts, mvp_data, mvp_scouters, sea_orm_active_enums::{Stations, TournamentLevels}, upcoming_game, upcoming_team
    },
    snowgrave::check_complete::CheckMatchErr,
};

#[derive(Debug, Clone)]
pub struct Six<T>(pub [T; 6]);

impl<T> TryFrom<Vec<T>> for Six<T> {
    type Error = CheckMatchErr;

    fn try_from(v: Vec<T>) -> Result<Self, Self::Error> {
        Ok(Six(
            v.try_into().map_err(|_| CheckMatchErr::Not6Teams)?
        ))
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TeamData {
    pub is_ab_team: bool,
    pub team: i32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Scouter {
    pub id: i32,
    pub scouter_id: Uuid,
    pub station: Stations,
    pub done: bool,
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct ScouterWithoutId {
    pub id: i32,
    pub station: Stations,
    pub done: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScouterWithScore {
    pub id: i32,
    pub scouter_id: Uuid,
    pub station: Stations,
    pub total_score: i32,
    pub done: bool,
}

impl From<&game_scouts::Model> for Scouter {
    fn from(m: &game_scouts::Model) -> Self {
        Self {
            id: m.id,
            scouter_id: m.scouter_id,
            station: m.station,
            done: m.done,
        }
    }
}

impl From<&game_scouts::Model> for ScouterWithoutId {
    fn from(m: &game_scouts::Model) -> Self {
        Self {
            id: m.id,
            station: m.station,
            done: m.done,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ScoutingTeam<S> {
    pub id: i32,
    pub station: Stations,
    pub team: TeamData,
    pub scouters: Vec<S>,
}

pub type ScoutingTeamThin = ScoutingTeam<Scouter>;
pub type ScoutingTeamThinWithoutId = ScoutingTeam<ScouterWithoutId>;
pub type ScoutingTeamFull = ScoutingTeam<ScouterWithScore>;

#[derive(Debug, Clone, Serialize)]
pub struct MvpScouter {
    pub id: i32,
    pub is_blue: bool,
    pub scouter_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct MvpData {
    pub id: i32,
    pub mvp_team: TeamData,
    pub is_blue: bool,
    pub comment: String,
    pub total_score: i32,
    pub penalty_score: i32,
}

#[derive(Debug, Clone)]
pub struct Mvp {
    pub scouter: MvpScouter,
    pub data: MvpData,
}

impl From<(mvp_scouters::Model, mvp_data::Model)> for Mvp {
    fn from((mvp, data): (mvp_scouters::Model, mvp_data::Model)) -> Self {
        Self {
            scouter: MvpScouter {
                id: mvp.id,
                scouter_id: mvp.scouter,
                is_blue: mvp.is_blue
            },
            data: MvpData {
                id: data.id,
                mvp_team: TeamData {
                    is_ab_team: data.mvp_is_ab_team,
                    team: data.mvp_team,
                },
                comment: data.comment,
                is_blue: data.is_blue,
                total_score: data.total_score,
                penalty_score: data.penalty_score,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Game<Teams, MvpState> {
    pub id: i32,
    pub event_code: String,
    pub match_id: i32,
    pub set: i32,
    pub tournament_level: TournamentLevels,
    pub teams: Teams,
    pub mvp: MvpState,
}

pub struct MvpPartial {
    pub red: Option<MvpScouter>,
    pub blue: Option<MvpScouter>,
}

pub type GamePartial =
    Game<Vec<ScoutingTeamThin>, MvpPartial>;


#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct MvpIds {
    pub blue: Option<i32>,
    pub red: Option<i32>,
}

pub type GamePartialWithoutId = //   MVP ID ->
    Game<Vec<ScoutingTeamThinWithoutId>, MvpIds>;


impl GamePartial {
    pub async fn from_game_id(
        game_id: i32,
        db: &DatabaseConnection,
    ) -> Result<Self, DbErr> {
        // Fetch the game
        let game = upcoming_game::Entity::find_by_id(game_id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Game not found".into()))?;

        // Fetch teams (just IDs / thin info for partial)
        let teams_models = upcoming_team::Entity::find()
            .filter(upcoming_team::Column::GameId.eq(game_id))
            .all(db)
            .await?;

        // Convert to ScoutingTeamThin
        let teams: Vec<ScoutingTeamThin> = teams_models
            .into_iter()
            .map(|t| ScoutingTeamThin {
                id: t.id,
                station: t.station,
                team: crate::snowgrave::datatypes::TeamData {
                    is_ab_team: t.is_ab_team,
                    team: t.team,
                },
                scouters: vec![], // leave empty for partial
            })
            .collect();

        Ok(GamePartial {
            id: game.id,
            event_code: game.event_code,
            match_id: game.match_id,
            set: game.set,
            tournament_level: game.tournament_level,
            teams,         // matches Game<Vec<ScoutingTeamThin>, Option<MvpScouter>>
            mvp: MvpPartial { red: None, blue: None },     // fill later in hydrate_game
        })
    }
}

pub struct FullMvp {
    pub blue: Mvp,
    pub red: Mvp
}

pub type GameFull =
    Game<Six<ScoutingTeamFull>, FullMvp>;

pub fn build_teams_thin(
    teams: Vec<upcoming_team::Model>,
    scouts: HashMap<i32, Vec<game_scouts::Model>>,
) -> Result<Six<ScoutingTeamThin>, CheckMatchErr> {
    let teams = teams
        .into_iter()
        .map(|team| {
            let scouters = scouts
                .get(&team.id)
                .ok_or(CheckMatchErr::ThereIsNotOneScouterPerTeam)?
                .iter()
                .map(Scouter::from)
                .collect();

            Ok(ScoutingTeam {
                id: team.id,
                station: team.station,
                team: TeamData {
                    is_ab_team: team.is_ab_team,
                    team: team.team,
                },
                scouters,
            })
        })
        .collect::<Result<Vec<_>, CheckMatchErr>>()?;

    teams.try_into()
}

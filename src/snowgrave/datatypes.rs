// =========================
// Domain + SeaORM Boundary
// =========================

use std::collections::HashMap;
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Serialize)]
pub struct ScoutingTeam<S> {
    pub id: i32,
    pub station: Stations,
    pub team: TeamData,
    pub scouters: Vec<S>,
}

pub type ScoutingTeamThin = ScoutingTeam<Scouter>;
pub type ScoutingTeamFull = ScoutingTeam<ScouterWithScore>;

#[derive(Debug, Clone, Serialize)]
pub struct MvpScouter {
    pub id: i32,
    pub scouter_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct MvpData {
    pub id: i32,
    pub mvp_team: TeamData,
    pub comment: String,
    pub total_score_for_red: i32,
    pub total_score_for_blue: i32,
    pub penalty_score_for_red: i32,
    pub penalty_score_for_blue: i32,
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
            },
            data: MvpData {
                id: data.id,
                mvp_team: TeamData {
                    is_ab_team: data.mvp_is_ab_team,
                    team: data.mvp_team,
                },
                comment: data.comment,
                total_score_for_red: data.total_score_for_red,
                total_score_for_blue: data.total_score_for_blue,
                penalty_score_for_red: data.penalty_score_for_red,
                penalty_score_for_blue: data.penalty_score_for_blue,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Game<Teams, MvpState> {
    pub id: i32,
    pub event_code: String,
    pub match_id: i32,
    pub set: i32,
    pub tournament_level: TournamentLevels,
    pub teams: Teams,
    pub mvp: MvpState,
}


pub type GamePartial =
    Game<Vec<ScoutingTeamThin>, Option<MvpScouter>>;


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
            mvp: None,     // fill later in hydrate_game
        })
    }
}


pub type GameFull =
    Game<Six<ScoutingTeamFull>, Mvp>;

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

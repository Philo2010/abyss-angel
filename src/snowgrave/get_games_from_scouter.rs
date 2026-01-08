use std::collections::HashMap;

use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::{
    entity::{
        game_scouts,
        mvp_scouters,
        upcoming_game,
        upcoming_team,
    },
    snowgrave::datatypes::{
        GamePartial, GamePartialWithoutId, MvpScouter, Scouter, ScouterWithoutId, ScoutingTeamThin, ScoutingTeamThinWithoutId, TeamData
    },
};

pub async fn get_games_for_scouter(scouter: Uuid, db: &DatabaseConnection,) -> Result<Vec<GamePartialWithoutId>, DbErr> {
    let scout_entries: Vec<game_scouts::Model> =
        game_scouts::Entity::find()
            .filter(game_scouts::Column::ScouterId.eq(scouter))
            .filter(game_scouts::Column::Done.eq(false))
            .all(db)
            .await?;

    if scout_entries.is_empty() {
        return Ok(Vec::new());
    }

    let game_ids: Vec<i32> =
        scout_entries.iter().map(|s| s.game_id).collect();

    let team_ids: Vec<i32> =
        scout_entries.iter().map(|s| s.team_id).collect();

    let games: HashMap<i32, upcoming_game::Model> =
        upcoming_game::Entity::find()
            .filter(upcoming_game::Column::Id.is_in(game_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|g| (g.id, g))
            .collect();

    let teams: HashMap<i32, upcoming_team::Model> =
        upcoming_team::Entity::find()
            .filter(upcoming_team::Column::Id.is_in(team_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

    let mvps: HashMap<i32, mvp_scouters::Model> =
        mvp_scouters::Entity::find()
            .filter(mvp_scouters::Column::Scouter.eq(scouter))
            .all(db)
            .await?
            .into_iter()
            .map(|m| (m.id, m))
            .collect();

    let mut scouts_by_game_team: HashMap<(i32, i32), Vec<ScouterWithoutId>> =
        HashMap::new();

    for scout in scout_entries {
        scouts_by_game_team
            .entry((scout.game_id, scout.team_id))
            .or_default()
            .push(ScouterWithoutId::from(&scout));
    }

    let mut result = Vec::new();

    for game in games.values() {
        let teams_for_game: Vec<ScoutingTeamThinWithoutId> =
            scouts_by_game_team
                .iter()
                .filter(|((game_id, _), _)| *game_id == game.id)
                .map(|((_, team_id), scouters)| {
                    let team = teams
                        .get(team_id)
                        .expect("team cached earlier");

                    ScoutingTeamThinWithoutId {
                        id: team.id,
                        station: team.station,
                        team: TeamData {
                            is_ab_team: team.is_ab_team,
                            team: team.team,
                        },
                        scouters: scouters.clone(),
                    }
                })
                .collect();

        let mvp = game
            .mvp_id
            .and_then(|id| mvps.get(&id))
            .map(|m| m.id);

        result.push(GamePartialWithoutId {
            id: game.id,
            event_code: game.event_code.clone(),
            match_id: game.match_id,
            set: game.set,
            tournament_level: game.tournament_level,
            teams: teams_for_game,
            mvp,
        });
    }

    Ok(result)
}

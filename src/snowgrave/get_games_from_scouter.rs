use std::collections::HashMap;

use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
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
        GamePartialWithoutId, MvpIds, ScouterWithoutId,
        ScoutingTeamThinWithoutId, TeamData,
    },
};

pub async fn get_games_for_scouter(
    scouter: Uuid,
    db: &DatabaseConnection,
) -> Result<Vec<GamePartialWithoutId>, DbErr> {

    // -------------------------
    // 1. Active scouting entries
    // -------------------------
    let scout_entries: Vec<game_scouts::Model> =
        game_scouts::Entity::find()
            .filter(game_scouts::Column::ScouterId.eq(scouter))
            .filter(game_scouts::Column::Done.eq(false))
            .all(db)
            .await?;

    // -------------------------
    // 2. MVPs owned by scouter
    // -------------------------
    let mvps: HashMap<i32, mvp_scouters::Model> =
        mvp_scouters::Entity::find()
            .filter(mvp_scouters::Column::Scouter.eq(scouter))
            .filter(mvp_scouters::Column::Data.is_null())
            .all(db)
            .await?
            .into_iter()
            .map(|m| (m.id, m))
            .collect();

    let scout_game_ids: Vec<i32> =
        scout_entries.iter().map(|s| s.game_id).collect();

    let mvp_ids: Vec<i32> =
        mvps.keys().copied().collect();

    if scout_game_ids.is_empty() && mvp_ids.is_empty() {
        return Ok(Vec::new());
    }

    // -------------------------
    // 3. Fetch games (OR logic)
    // -------------------------
    let mut condition = Condition::any();

    if !scout_game_ids.is_empty() {
        condition = condition.add(
            upcoming_game::Column::Id.is_in(scout_game_ids)
        );
    }

    if !mvp_ids.is_empty() {
        condition = condition
            .add(upcoming_game::Column::MvpIdBlue.is_in(mvp_ids.clone()))
            .add(upcoming_game::Column::MvpIdRed.is_in(mvp_ids));
    }

    let games: HashMap<i32, upcoming_game::Model> =
        upcoming_game::Entity::find()
            .filter(condition)
            .all(db)
            .await?
            .into_iter()
            .map(|g| (g.id, g))
            .collect();

    // -------------------------
    // 4. Teams (scouting only)
    // -------------------------
    let team_ids: Vec<i32> =
        scout_entries.iter().map(|s| s.team_id).collect();

    let teams: HashMap<i32, upcoming_team::Model> =
        if team_ids.is_empty() {
            HashMap::new()
        } else {
            upcoming_team::Entity::find()
                .filter(upcoming_team::Column::Id.is_in(team_ids))
                .all(db)
                .await?
                .into_iter()
                .map(|t| (t.id, t))
                .collect()
        };

    // -------------------------
    // 5. Group scouters by (game, team)
    // -------------------------
    let mut scouts_by_game_team: HashMap<(i32, i32), Vec<ScouterWithoutId>> =
        HashMap::new();

    for scout in scout_entries {
        scouts_by_game_team
            .entry((scout.game_id, scout.team_id))
            .or_default()
            .push(ScouterWithoutId::from(&scout));
    }

    // -------------------------
    // 6. Build response
    // -------------------------
    let mut result = Vec::new();

    for game in games.values() {

        let teams_for_game: Vec<ScoutingTeamThinWithoutId> =
            scouts_by_game_team
                .iter()
                .filter(|((gid, _), _)| *gid == game.id)
                .map(|((_, team_id), scouters)| {
                    let team = teams
                        .get(team_id)
                        .expect("team referenced by scout must exist");

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

        let mut mvp_blue = None;
        let mut mvp_red = None;

        if let Some(id) = game.mvp_id_blue {
            if mvps.contains_key(&id) {
                mvp_blue = Some(id);
            }
        }

        if let Some(id) = game.mvp_id_red {
            if mvps.contains_key(&id) {
                mvp_red = Some(id);
            }
        }

        let mvp = MvpIds {
            blue: mvp_blue,
            red: mvp_red,
        };

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

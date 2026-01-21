use std::collections::HashMap;

use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::{entity::{game_scouts, genertic_header, mvp_data, mvp_scouters}, snowgrave::{check_complete::CheckMatchErr, datatypes::{FullMvp, GameFull, GamePartial, Mvp, MvpScouter, ScouterWithScore, ScoutingTeamFull, Six}}};

async fn fetch_scouts_for_game(
    game_id: i32,
    db: &DatabaseConnection,
) -> Result<HashMap<i32, Vec<ScouterWithScore>>, CheckMatchErr> {
    let scouts = game_scouts::Entity::find()
        .filter(game_scouts::Column::GameId.eq(game_id))
        .all(db)
        .await
        .map_err(CheckMatchErr::DbErr)?;

    let mut scouts_by_team: HashMap<i32, Vec<ScouterWithScore>> = HashMap::new();


    //TODO: Update to find total score
    //get the total score
        

    for scout in scouts {
        let scout_data_header = genertic_header::Entity::find()
        .filter(genertic_header::Column::SnowgraveScoutId.eq(scout.id)).one(db).await?.ok_or(DbErr::RecordNotFound("total score".to_string()))?;
        scouts_by_team
            .entry(scout.team_id)
            .or_default()
            .push(ScouterWithScore {
                id: scout.id,
                scouter_id: scout.scouter_id,
                station: scout.station,
                done: scout.done,
                is_redo: scout.is_redo,
                total_score: scout_data_header.total_score, // fill later if needed
            });
    }

    Ok(scouts_by_team)
}


async fn fetch_mvp_full(
    mvp_id: i32,
    db: &DatabaseConnection,
) -> Result<Mvp, CheckMatchErr> {
    let mvp_model = mvp_scouters::Entity::find_by_id(mvp_id)
        .one(db)
        .await
        .map_err(CheckMatchErr::DbErr)?
        .ok_or(CheckMatchErr::ThereIsNoMVP)?;

    let data_id = mvp_model.data.ok_or(CheckMatchErr::MvpIsNotDone)?;
    let data_model = mvp_data::Entity::find_by_id(data_id)
        .one(db)
        .await
        .map_err(CheckMatchErr::DbErr)?
        .ok_or(CheckMatchErr::MvpIsNotDone)?;

    Ok(Mvp {
        scouter: MvpScouter {
            id: mvp_model.id,
            scouter_id: mvp_model.scouter,
            is_blue: mvp_model.is_blue
        },
        data: crate::snowgrave::datatypes::MvpData {
            id: data_model.id,
            mvp_team: crate::snowgrave::datatypes::TeamData {
                is_ab_team: data_model.mvp_is_ab_team,
                team: data_model.mvp_team,
            },
            comment: data_model.comment,
            total_score: data_model.total_score,
            penalty_score: data_model.penalty_score,
            is_blue: data_model.is_blue,
        },
    })
}



pub async fn hydrate_game(
    partial: GamePartial,
    db: &DatabaseConnection,
) -> Result<Option<GameFull>, CheckMatchErr> {
    // fetch all scouts for each team
    let scouts_by_team: HashMap<i32, Vec<ScouterWithScore>> = fetch_scouts_for_game(partial.id, db).await?;

    // check if any team is missing scouts
    if partial.teams.len() != 6 || scouts_by_team.len() != 6 {
        return Ok(None); // game not ready
    }

    // check if all scouts are done
    if scouts_by_team.values().any(|sc| sc.iter().any(|s| !s.done)) {
        return Ok(None); // not ready
    }


    

    // fetch MVP
    let mvp_full_red = match &partial.mvp.red {
        Some(m) => {
            let res = fetch_mvp_full(m.id, db).await?;
            res
        },
        None => return Ok(None),
    };

    let mvp_full_blue = match &partial.mvp.blue {
        Some(m) => {
            let res = fetch_mvp_full(m.id, db).await?;
            res
        },
        None => return Ok(None),
    };

    // assemble full teams
    let teams_full: Six<ScoutingTeamFull> = partial.teams
        .into_iter()
        .map(|t| {
            let scouters = scouts_by_team.get(&t.id).unwrap().clone();
            ScoutingTeamFull { id: t.id, station: t.station, team: t.team, scouters }
        })
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| CheckMatchErr::Not6Teams)?;

    Ok(Some(GameFull {
        id: partial.id,
        event_code: partial.event_code,
        match_id: partial.match_id,
        set: partial.set,
        tournament_level: partial.tournament_level,
        teams: teams_full,
        mvp: FullMvp {
            blue: mvp_full_blue,
            red: mvp_full_red,
        },
    }))
}

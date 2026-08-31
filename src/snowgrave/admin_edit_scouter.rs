// Admin edit for scouting data, at any stage.
//
// Kept deliberately separate from the insert/check/publish pipeline: once a
// scout's data is published (a genertic_header row exists for it), this
// patches that row and its year-specific row directly in place and never
// calls check()/publish() — publish() is a blind insert, so re-running it on
// already-published data would create a duplicate genertic_header row and
// double-count that scout everywhere downstream. If nothing has published
// yet (still pending, or flagged for redo), it's safe to reuse the existing
// edit-and-recheck pipeline unchanged.

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder,
};

use crate::{
    SETTINGS,
    backenddb::game::game_dispatch,
    entity::{game_scouts, genertic_header, scout_game_midway_insert},
    snowgrave::snowgrave_edit_scouter::{edit_scouter, EditSnow},
};

pub async fn edit_scouting_data(db: &DatabaseConnection, edit_data: EditSnow) -> Result<(), DbErr> {
    let snowgrave_scout = game_scouts::Entity::find_by_id(edit_data.snowgrave_scout_id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("Could not find scout!".to_string()))?;

    let midway_id = snowgrave_scout
        .game_midway
        .ok_or_else(|| DbErr::Custom("Scout has no submitted data to edit".to_string()))?;
    let midway = scout_game_midway_insert::Entity::find_by_id(midway_id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::Custom("Midway record not found".to_string()))?;

    let existing_header = genertic_header::Entity::find()
        .filter(genertic_header::Column::EventCode.eq(midway.event_code.clone()))
        .filter(genertic_header::Column::MatchId.eq(midway.match_id))
        .filter(genertic_header::Column::Set.eq(midway.set))
        .filter(genertic_header::Column::TournamentLevel.eq(midway.tournament_level))
        .filter(genertic_header::Column::Team.eq(midway.team))
        .filter(genertic_header::Column::IsAbTeam.eq(midway.is_ab_team))
        .filter(genertic_header::Column::Station.eq(midway.station))
        .filter(genertic_header::Column::GameTypeId.eq(midway.game_type_id))
        .order_by_desc(genertic_header::Column::Id)
        .one(db)
        .await?;

    match existing_header {
        // Already checked and published — patch the live row directly.
        Some(header) => edit_published_data(&header, edit_data, db).await,
        // Not published yet (still pending, or flagged for redo) — the
        // ordinary edit-and-recheck pipeline is safe here since nothing has
        // published for this scout yet.
        None => edit_scouter(edit_data, db).await,
    }
}

async fn edit_published_data(
    header: &genertic_header::Model,
    edit_data: EditSnow,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let game_funcion = game_dispatch(SETTINGS.year);
    let res = game_funcion.edit(header.game_id, edit_data.game, db).await?;

    let mut header_active: genertic_header::ActiveModel = header.clone().into();
    header_active.total_score = Set(res.total_score);
    header_active.teleop_score = Set(res.teleop_score);
    header_active.auto_score = Set(res.auto_score);
    if let Some(defence) = edit_data.defence {
        header_active.defence = Set(defence);
    }
    if let Some(comment) = edit_data.comment {
        header_active.comment = Set(comment);
    }
    if let Some(defence_main) = edit_data.defence_main {
        header_active.defence_main = Set(defence_main);
    }
    if let Some(defence_target) = edit_data.defence_target {
        header_active.defence_target = Set(defence_target);
    }
    if let Some(auto_time) = edit_data.auto_time {
        header_active.auto_time = Set(auto_time);
    }
    if let Some(dead) = edit_data.dead {
        header_active.dead = Set(dead);
    }
    if let Some(dnf) = edit_data.dnf {
        header_active.dnf = Set(dnf);
    }

    header_active.update(db).await?;

    Ok(())
}

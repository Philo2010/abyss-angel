// Admin edit for MVP scouting data.
//
// Simpler than admin_edit_scouter: an MVP report never gets published into
// its own genertic_header row — it only feeds the is_mvp flag and the
// score-range check on whichever station rows do publish — so there's no
// published-row duplication risk to guard against here. This always upserts
// mvp_data in place (insert on first submission, update thereafter) and
// reruns check() the same way insert_mvp_data does.

use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};

use crate::{
    entity::{mvp_data, mvp_scouters, upcoming_game},
    snowgrave::check_system::check,
    snowgrave::insert_mvp_data::MvpInsert,
};

pub async fn edit_mvp_data(data: MvpInsert, db: &DatabaseConnection) -> Result<(), DbErr> {
    let mvp_mod = mvp_scouters::Entity::find_by_id(data.mvp_id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("Could not find scouter!".to_string()))?;

    match mvp_mod.data {
        Some(existing_id) => {
            let existing = mvp_data::Entity::find_by_id(existing_id)
                .one(db)
                .await?
                .ok_or_else(|| DbErr::RecordNotFound("Could not find mvp data!".to_string()))?;
            let mut active: mvp_data::ActiveModel = existing.into();
            active.mvp_team = Set(data.mvp_team.number);
            active.mvp_is_ab_team = Set(data.mvp_team.is_ab_team);
            active.comment = Set(data.comment);
            active.total_score = Set(data.total_score);
            active.penalty_score = Set(data.penalty_score);
            active.update(db).await?;
        }
        None => {
            let mvp_insert = mvp_data::ActiveModel {
                id: NotSet,
                mvp_team: Set(data.mvp_team.number),
                mvp_is_ab_team: Set(data.mvp_team.is_ab_team),
                comment: Set(data.comment),
                total_score: Set(data.total_score),
                penalty_score: Set(data.penalty_score),
                is_blue: Set(mvp_mod.is_blue),
            };
            let data_id = mvp_insert.insert(db).await?;

            let mut mvp: mvp_scouters::ActiveModel = mvp_mod.clone().into();
            mvp.data = Set(Some(data_id.id));
            mvp.update(db).await?;
        }
    }

    let game_id = if mvp_mod.is_blue {
        upcoming_game::Entity::find()
            .filter(upcoming_game::Column::MvpIdBlue.eq(data.mvp_id))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("Could not find game!".to_string()))?
    } else {
        upcoming_game::Entity::find()
            .filter(upcoming_game::Column::MvpIdRed.eq(data.mvp_id))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("Could not find game!".to_string()))?
    };

    check(game_id.id, db).await?;

    Ok(())
}

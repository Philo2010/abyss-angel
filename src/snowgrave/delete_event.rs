use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, TransactionTrait,
    prelude::Expr,
};

use crate::entity::{
    game_scouts, mvp_scouters, pit_upcoming, scout_game_midway_insert, upcoming_game,
};

pub async fn delete_event(event_code: &str, db: &DatabaseConnection) -> Result<(), DbErr> {
    let txn = db.begin().await?;

    // Find all games for this event
    let games = upcoming_game::Entity::find()
        .filter(upcoming_game::Column::EventCode.eq(event_code))
        .all(&txn)
        .await?;

    let game_ids: Vec<i32> = games.iter().map(|g| g.id).collect();

    // Collect mvp_scouter IDs referenced by games
    let mvp_scouter_ids: Vec<i32> = games
        .iter()
        .flat_map(|g| [g.mvp_id_blue, g.mvp_id_red])
        .flatten()
        .collect();

    // Find game_scouts for these games, collect game_midway IDs
    let midway_ids: Vec<i32> = if !game_ids.is_empty() {
        game_scouts::Entity::find()
            .filter(game_scouts::Column::GameId.is_in(game_ids.clone()))
            .all(&txn)
            .await?
            .iter()
            .filter_map(|s| s.game_midway)
            .collect()
    } else {
        vec![]
    };

    // Null out upcoming_game mvp FKs so we can delete upcoming_game
    upcoming_game::Entity::update_many()
        .col_expr(upcoming_game::Column::MvpIdBlue, Expr::value(Option::<i32>::None))
        .col_expr(upcoming_game::Column::MvpIdRed, Expr::value(Option::<i32>::None))
        .filter(upcoming_game::Column::EventCode.eq(event_code))
        .exec(&txn)
        .await?;

    // Delete game_scouts
    if !game_ids.is_empty() {
        game_scouts::Entity::delete_many()
            .filter(game_scouts::Column::GameId.is_in(game_ids.clone()))
            .exec(&txn)
            .await?;
    }

    // Delete submitted scout data (mid_header rows)
    if !midway_ids.is_empty() {
        scout_game_midway_insert::Entity::delete_many()
            .filter(scout_game_midway_insert::Column::Id.is_in(midway_ids))
            .exec(&txn)
            .await?;
    }

    // Delete only mvp_scouters that have no submitted data
    if !mvp_scouter_ids.is_empty() {
        mvp_scouters::Entity::delete_many()
            .filter(mvp_scouters::Column::Id.is_in(mvp_scouter_ids))
            .filter(mvp_scouters::Column::Data.is_null())
            .exec(&txn)
            .await?;
    }

    // Delete upcoming_game — cascades to upcoming_team
    upcoming_game::Entity::delete_many()
        .filter(upcoming_game::Column::EventCode.eq(event_code))
        .exec(&txn)
        .await?;

    // Delete pit_upcoming assignments (not actual pit data)
    pit_upcoming::Entity::delete_many()
        .filter(pit_upcoming::Column::EventCode.eq(event_code))
        .exec(&txn)
        .await?;

    txn.commit().await?;
    Ok(())
}

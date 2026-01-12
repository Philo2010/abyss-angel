use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, prelude::Expr};
use uuid::Uuid;

use crate::entity::{game_scouts, mvp_scouters};



pub async fn sub_scout(db: &DatabaseConnection, og: &Uuid, replacement: &Uuid) -> Result<(), DbErr> {
    game_scouts::Entity::update_many()
        .col_expr(game_scouts::Column::ScouterId, Expr::value(*replacement))
        .filter(game_scouts::Column::ScouterId.eq(*og))
        .exec(db).await?;

    mvp_scouters::Entity::update_many()
        .col_expr(game_scouts::Column::ScouterId, Expr::val(*replacement))
        .filter(game_scouts::Column::ScouterId.eq(*og))
        .exec(db).await?;

    Ok(())
}
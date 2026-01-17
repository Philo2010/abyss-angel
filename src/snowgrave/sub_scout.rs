use std::collections::HashMap;

use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityOrSelect, EntityTrait, QueryFilter, prelude::Expr};
use uuid::Uuid;

use crate::entity::{game_scouts, mvp_scouters};



pub async fn sub_scout(
    db: &DatabaseConnection,
    og: &Uuid,
    replacement: &Uuid,
) -> Result<(), DbErr> {
    game_scouts::Entity::update_many()
        .col_expr(
            game_scouts::Column::ScouterId,
            Expr::value(*replacement),
        )
        .filter(game_scouts::Column::ScouterId.eq(*og))
        .filter(game_scouts::Column::Done.eq(false))
        .exec(db)
        .await?;


    mvp_scouters::Entity::update_many()
        .col_expr(
            mvp_scouters::Column::Scouter,
            Expr::value(*replacement),
        )
        .filter(mvp_scouters::Column::Scouter.eq(*og))
        .filter(mvp_scouters::Column::Data.is_null())
        .exec(db)
        .await?;

    Ok(())
}
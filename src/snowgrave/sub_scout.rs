
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, prelude::Expr};
use uuid::Uuid;

use crate::entity::{game_scouts, mvp_scouters};



pub async fn sub_scout(
    db: &DatabaseConnection,
    og: &Uuid,
    replacement: &Uuid,
    game_ids: Option<&[i32]>,
) -> Result<(), DbErr> {
    let mut query = game_scouts::Entity::update_many()
        .col_expr(
            game_scouts::Column::ScouterId,
            Expr::value(*replacement),
        )
        .filter(game_scouts::Column::ScouterId.eq(*og))
        .filter(game_scouts::Column::Done.eq(false))
        .filter(game_scouts::Column::GameMidway.is_null());

    if let Some(ids) = game_ids {
        query = query.filter(game_scouts::Column::GameId.is_in(ids.iter().copied()));
    }

    query.exec(db).await?;

    // Only replace MVP scouters when doing a full sub (no game_ids filter)
    if game_ids.is_none() {
        mvp_scouters::Entity::update_many()
            .col_expr(
                mvp_scouters::Column::Scouter,
                Expr::value(*replacement),
            )
            .filter(mvp_scouters::Column::Scouter.eq(*og))
            .filter(mvp_scouters::Column::Data.is_null())
            .exec(db)
            .await?;
    }

    Ok(())
}
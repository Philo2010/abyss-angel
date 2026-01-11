use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect, TransactionError, TransactionTrait, prelude::Expr};

use crate::{backenddb::example_game::ActiveModel, entity::{game_scouts, genertic_header, upcoming_team, warning}, scoutwarn::{self, send_warning::SendWarning}, snowgrave::check::CheckFailerReturn};

pub async fn cast_snowgrave(
    game_id: i32,
    fails: CheckFailerReturn,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            let team_ids: Vec<i32> = upcoming_team::Entity::find()
                .select_only()
                .column(upcoming_team::Column::Id)
                .filter(upcoming_team::Column::GameId.eq(game_id))
                .into_tuple()
                .all(txn)
                .await?;

                for scouter in fails.reasons {
                    //send the warning to them
                    let warn = SendWarning {
                        sender: None, //We are snowgrave
                        receiver: scouter.scouter_id,
                        message: format!("AUTOMATED SNOWGRAVE WARNING:\nWE HAVE FOUND AN ERROR IN YOUR SCOUT PACKAGE OF GAME {}.\n PLEASE IMPROVE FOR LATER, SYSTEMS DO NOT LIKE TO SLOW DUE TO HUMAN ERROR. -SG", fails.game_number),
                    };
                    scoutwarn::send_warning::send_warning(warn, txn).await?;

                    //now to mark them as not done and redo
                    let mut res: game_scouts::ActiveModel = game_scouts::Entity::find_by_id(scouter.id).one(txn).await?.ok_or(DbErr::RecordNotFound("Could not find scouter!".to_string()))?.into();
                    res.is_redo = Set(true);
                    res.done = Set(false);
                    res.update(txn).await?;
                }

            //for reason in fails.reasons

            genertic_header::Entity::update_many()
                .filter(genertic_header::Column::SnowgraveScoutId.is_in(team_ids))
                .col_expr(genertic_header::Column::IsPending, Expr::value(false))
                .exec(txn)
                .await?;

            let redo_scout_ids: Vec<i32> = game_scouts::Entity::find()
                .select_only()
                .column(game_scouts::Column::Id)
                .filter(game_scouts::Column::GameId.eq(game_id))
                .filter(game_scouts::Column::TeamId.is_in(fails.teams_to_redo))
                .into_tuple()
                .all(txn)
                .await?;

            if !redo_scout_ids.is_empty() {
                genertic_header::Entity::update_many()
                    .filter(genertic_header::Column::SnowgraveScoutId.is_in(redo_scout_ids))
                    .col_expr(genertic_header::Column::IsMarked, Expr::value(true))
                    .exec(txn)
                    .await?;
            }

            Ok(())
        })
    }).await
    .map_err(|e| match e {
        TransactionError::Connection(err) => err,
        TransactionError::Transaction(err) => err,
    })?;

    Ok(())
}

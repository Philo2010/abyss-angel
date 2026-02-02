use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect, TransactionError, TransactionTrait, prelude::Expr};

use crate::{ entity::{game_scouts, genertic_header, upcoming_team, warning}, scoutwarn::{self, send_warning::SendWarning}, snowgrave::check::CheckFailerReturn};

pub async fn cast_snowgrave(
    game_id: i32,
    fails: CheckFailerReturn,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    use std::collections::HashSet;

    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {

            let CheckFailerReturn {
                game_number,
                teams_to_redo,
                scouts_to_forgive,
                reasons,
                winner_teams,
            } = fails;

            let fail_ids: HashSet<i32> = reasons.iter().map(|s| s.id).collect();
            let forgive_ids: HashSet<i32> = scouts_to_forgive.iter().map(|s| s.id).collect();
            let winner_ids: HashSet<i32> = winner_teams.into_iter().collect();

            // ---------- INVARIANTS ----------
            debug_assert!(
                fail_ids.is_disjoint(&forgive_ids),
                "Scout cannot be both failed and forgiven"
            );

            // ---------- WARN + REDO FAILED SCOUTS ----------
            for scouter in &reasons {
                let warn = SendWarning {
                    sender: None,
                    receiver: scouter.scouter_id,
                    message: format!(
                        "AUTOMATED SNOWGRAVE WARNING:\n\
                         ERROR FOUND IN GAME {}.\n\
                         PLEASE IMPROVE FOR LATER. -SG",
                        game_number
                    ),
                };
                scoutwarn::send_warning::send_warning(warn, txn).await?;

                let mut scout: game_scouts::ActiveModel =
                    game_scouts::Entity::find_by_id(scouter.id)
                        .one(txn)
                        .await?
                        .ok_or(DbErr::RecordNotFound("Missing scout".into()))?
                        .into();

                scout.is_redo = Set(true);
                scout.done = Set(false);
                scout.update(txn).await?;
            }

            // ---------- MARK REDO HEADERS ----------
            let redo_scout_ids: Vec<i32> = game_scouts::Entity::find()
                .select_only()
                .column(game_scouts::Column::Id)
                .filter(game_scouts::Column::GameId.eq(game_id))
                .filter(game_scouts::Column::TeamId.is_in(teams_to_redo))
                .into_tuple()
                .all(txn)
                .await?;

            if !redo_scout_ids.is_empty() {
                genertic_header::Entity::update_many()
                    .filter(genertic_header::Column::SnowgraveScoutId.is_in(redo_scout_ids))
                    .col_expr(genertic_header::Column::IsPending, Expr::value(false))
                    .col_expr(genertic_header::Column::IsMarked, Expr::value(true))
                    // ❌ Leave IsDup alone!
                    .exec(txn)
                    .await?;
            }

            // ---------- FORGIVEN SCOUTS ----------
            if !forgive_ids.is_empty() {
                genertic_header::Entity::update_many()
                    .filter(genertic_header::Column::SnowgraveScoutId.is_in(forgive_ids.iter().copied()))
                    .col_expr(genertic_header::Column::IsPending, Expr::value(false))
                    .col_expr(genertic_header::Column::IsMarked, Expr::value(false))
                    // ❌ Leave IsDup alone!
                    .exec(txn)
                    .await?;
            }

            // ---------- WINNERS (clear duplicate flag) ----------
            if !winner_ids.is_empty() {
                genertic_header::Entity::update_many()
                    .filter(genertic_header::Column::SnowgraveScoutId.is_in(winner_ids.iter().copied()))
                    .col_expr(genertic_header::Column::IsDup, Expr::value(false))
                    .exec(txn)
                    .await?;
            }

            // ---------- FINAL SAFETY ASSERT ----------
            #[cfg(debug_assertions)]
            {
                let bad_headers = genertic_header::Entity::find()
                    .filter(genertic_header::Column::GameId.eq(game_id))
                    .filter(genertic_header::Column::IsPending.eq(false))
                    .filter(genertic_header::Column::IsMarked.eq(false))
                    .filter(genertic_header::Column::SnowgraveScoutId.is_not_in(forgive_ids.iter().copied()))
                    .all(txn)
                    .await?;

                debug_assert!(
                    bad_headers.is_empty(),
                    "Header left resolved but neither marked nor forgiven (dup is allowed)"
                );
            }

            Ok(())
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(err) => err,
        TransactionError::Transaction(err) => err,
    })?;

    Ok(())
}

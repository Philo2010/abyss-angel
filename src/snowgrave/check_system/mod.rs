use sea_orm::{DatabaseConnection, DbErr};

use crate::snowgrave::check_system::db_work::{publish, punish};

pub mod check_bind;
pub mod check_if_filled;
pub mod check_mid;
pub mod precheck;
pub mod db_work;
pub mod nobonoko;

// ── Check system configuration ────────────────────────────────────────────────
//
// All tunable thresholds live here so they can be adjusted in one place.

/// Maximum total "spread" (sum of gaps between adjacent sorted scores) that the
/// tightest cluster of scout scores may span before a scout is considered an
/// outlier. Used by `find_point_distance::check_pass`.
pub const CLUSTER_MAX_SPREAD: i32 = 75;

/// If the fraction of outlier scouts at a position is at or above this ratio,
/// the entire position is considered too corrupt to average and is failed
/// outright rather than averaged. Used by `precheck`.
pub const OUTLIER_FAIL_RATIO: f32 = 0.8;

/// How many points the summed scout scores for an alliance are allowed to
/// deviate from the MVP-reported score before the alliance is flagged as
/// failing the mid-game check. Used by `check_mid`.
pub const SCORE_RANGE_TOLERANCE: f32 = 0.60; // 60% of MVP score

//make scout range toleance
//auto fill MVP
//fix the weaird label light mode
//add a clear button

pub async fn check(upcoming_game_id: i32,db: &DatabaseConnection) -> Result<(), DbErr> {
    let res = check_bind::check_bind(upcoming_game_id, db).await?;
    match res {
        check_bind::CheckBindReturn::Passed(items, failer_infos) => {
            if !failer_infos.is_empty() {
                punish(failer_infos, db).await?;
            }
            publish(items, db).await?;
        },
        check_bind::CheckBindReturn::Failed(failer_infos) => {
            punish(failer_infos, db).await?;
        },
        check_bind::CheckBindReturn::NotDone => {
            //oh well, next time
            return Ok(());
        },
    }


    Ok(())
}
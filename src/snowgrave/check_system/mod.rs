use sea_orm::{DatabaseConnection, DbErr};

use crate::snowgrave::check_system::db_work::{publish, punish};

pub mod check_bind;
pub mod check_if_filled;
pub mod check_mid;
pub mod precheck;
pub mod db_work;


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
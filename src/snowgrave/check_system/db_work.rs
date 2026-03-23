use std::any;

use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect, prelude::Expr};

use crate::{backenddb::game::{GamesInserts, insert_game}, entity::game_scouts, scoutwarn::send_warning::{SendWarning, send_warning}, snowgrave::{check_system::check_bind::CheckBindReturn, datatypes::FailerInfo}};



pub enum CheckBindReturnSafe {
    Passed(Vec<GamesInserts>, Vec<FailerInfo>),
    Failed(Vec<FailerInfo>),
}

pub async fn publish(data: Vec<GamesInserts>, db: &DatabaseConnection) -> Result<(), DbErr> {

    for game in data {
        let _res = insert_game(&game, db).await?;
    }

    Ok(())
}

pub async fn punish(info: Vec<FailerInfo>, db: &DatabaseConnection) -> Result<(), DbErr> {


    //set the following games as failed
    game_scouts::Entity::update_many()
    .col_expr(game_scouts::Column::Done, Expr::value(false))
    .col_expr(game_scouts::Column::IsRedo, Expr::value(true))
    .filter(game_scouts::Column::Id.is_in(
        info.iter().map(|x| x.upcoming_scout_id).collect::<Vec<_>>()
    ))
    .exec(db)
    .await?;



    //send warnings
    let warns: Vec<SendWarning> = info.into_iter().map(|x| 
    SendWarning {
        sender: None,
        receiver: x.name,
        message: format!("Hello, you made an issue with team {} that is on station {}, please redo.", x.team.number, x.station),
    }).collect();

    for warn in warns {
        let res = send_warning(warn, db).await?;
    }

    Ok(())
    
}
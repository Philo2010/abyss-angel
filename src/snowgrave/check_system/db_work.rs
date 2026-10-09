use std::any;
use std::collections::HashSet;

use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect, prelude::Expr};

use crate::{SETTINGS, backenddb::game::{GamesInserts, insert_game}, entity::game_scouts, scoutwarn::send_warning::{SendWarning, send_warning}, snowgrave::{check_system::check_bind::CheckBindReturn, datatypes::FailerInfo}};



pub enum CheckBindReturnSafe {
    Passed(Vec<GamesInserts>, Vec<FailerInfo>),
    Failed(Vec<FailerInfo>),
}

pub async fn publish(data: Vec<GamesInserts>, db: &DatabaseConnection) -> Result<(), DbErr> {

    // Collect the events being finalized, then once every new row is in,
    // refresh DPDG for each from the latest event averages. This supersedes the
    // frozen per-match stamp so all of an event's rows stay consistent; the
    // scan is limited to the event(s) written here. Only changed rows are
    // written (guarded inside the refresh).
    let mut events: HashSet<String> = HashSet::new();
    for game in data {
        events.insert(game.header.event_code.clone());
        let _res = insert_game(&game, db).await?;
    }

    for event in events {
        super::dpdg_refresh::refresh_event_dpdg(&event, SETTINGS.year, db).await?;
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
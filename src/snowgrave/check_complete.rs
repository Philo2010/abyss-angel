use std::collections::HashMap;

use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use crate::{
    entity::{game_scouts, mvp_data, mvp_scouters, upcoming_game, upcoming_team},
    snowgrave::{cast_snowgrave, check, datatypes::{
        GameFull, GamePartial, Mvp, MvpScouter, ScouterWithScore, ScoutingTeamFull, Six, TeamData
    }, hydrate::hydrate_game},
};

pub enum CheckMatchErr {
    NotAllScoutersAreDone,
    MvpIsNotDone,
    ThereIsNotOneScouterPerTeam,
    ThereIsNoMVP,
    NoMatch,
    Not6Teams,
    DbErr(DbErr),
}

impl From<DbErr> for CheckMatchErr {
    fn from(e: DbErr) -> Self {
        CheckMatchErr::DbErr(e)
    }
}

/// Checks if a game is fully complete
/// Returns GameFull if everything exists
pub async fn check_match(
    game_id: i32,
    db: &DatabaseConnection,
) -> Result<(), CheckMatchErr> {
    let partial_game = GamePartial::from_game_id(game_id, db).await?;

    let game_full = match hydrate_game(partial_game, db).await? {
        Some(full) => full,             // all data is present
        None => return Err(CheckMatchErr::NotAllScoutersAreDone), // or another "not ready" error
    };
    let res = check::check(&game_full)?;

    let true_res = cast_snowgrave::cast_snowgrave(game_id, res, db).await?;

    return Ok(true_res);
}


use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::entity::sea_orm_active_enums::TournamentLevels;
use crate::snowgrave::blue::{self};
use crate::entity::{upcoming_game, upcoming_team};
use crate::snowgrave::snowgrave_que::{into_snow, into_snow_team};
use crate::snowgrave::snowgrave_que::Blue2DBErr;


pub async fn queue_snow(games: Vec<blue::TbaMatch>, event_code: &String, db: &DatabaseConnection) -> Result<(), Blue2DBErr> {
    let mut playoff_games: Vec<&blue::TbaMatch> = Vec::new();
    let mut games_ids: Vec<i32> = Vec::new();

    for game in &games {
        let res = into_snow(game, event_code)?;
        if res.tournament_level.clone().unwrap() == TournamentLevels::QualificationMatch {
            continue;
        }

        // Skip if this match already exists
        let existing = upcoming_game::Entity::find()
            .filter(upcoming_game::Column::EventCode.eq(event_code))
            .filter(upcoming_game::Column::MatchId.eq(game.match_number))
            .filter(upcoming_game::Column::Set.eq(game.set_number))
            .filter(upcoming_game::Column::TournamentLevel.eq(res.tournament_level.clone().unwrap()))
            .one(db)
            .await
            .map_err(Blue2DBErr::DbErr)?;

        if existing.is_some() {
            continue;
        }

        let db_res = match upcoming_game::Entity::insert(res).exec(db).await {
            Ok(a) => a,
            Err(a) => {
                return Err(Blue2DBErr::DbErr(a));
            }
        }.last_insert_id;

        playoff_games.push(game);
        games_ids.push(db_res);
    }

    for (game, id) in playoff_games.iter().zip(games_ids.iter()) {
        let res = into_snow_team(game, *id)?;
        match upcoming_team::Entity::insert_many(res).exec(db).await {
            Ok(_) => {},
            Err(a) => {
                return Err(Blue2DBErr::DbErr(a));
            },
        };
    }

    Ok(())
}

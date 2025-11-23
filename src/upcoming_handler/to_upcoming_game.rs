use std::{error::Error, num::ParseIntError};

use rocket::figment::value;
use sea_orm::{ActiveValue::{NotSet, Set}, DatabaseConnection, DbErr, EntityTrait};

use crate::{models::dyn_settings::ActiveModel, upcoming_handler::{blue::TbaMatch, upcoming_game, upcoming_team}};

pub enum UpcomingGameError {
    Database(DbErr),
    Parse(ParseIntError),
}

pub async fn insert_upcoming_game(db: &DatabaseConnection, value: &TbaMatch, event: &String) -> Result<(), UpcomingGameError> {

    let game = upcoming_game::ActiveModel {
        id: NotSet,
        event_code: Set(event.clone()),
        match_number: Set(value.match_number),
        set_number: Set(value.set_number),
        tournament_level: Set(value.comp_level.clone()),
    };

    let id = match upcoming_game::Entity::insert(game).exec(db).await {
        Ok(a) => a,
        Err(a) => {return Err(UpcomingGameError::Database(a));},
    };

    let mut team = Vec::with_capacity(6);

    for (i, name) in value.alliances.blue.team_keys.iter().enumerate() {

        let team_number: i32 = match ("99".to_owned() + name).parse() {
            Ok(a) => a,
            Err(a) => {
                return Err(UpcomingGameError::Parse(a));
            }
        };

        team.push(upcoming_team::ActiveModel {
            id: NotSet,
            station: Set(format!("Blue {i}")),
            team: Set(team_number),
            scouter: NotSet,
            game_id: Set(id.last_insert_id),
        });
    }

    match upcoming_team::Entity::insert_many(team).exec(db).await {
        Ok(_) => {},
        Err(a) => {return Err(UpcomingGameError::Database(a));},
    };


    Ok(())
}
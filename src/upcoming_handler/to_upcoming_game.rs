use std::{error::Error, num::ParseIntError};

use rocket::figment::value;
use sea_orm::{ActiveValue::{NotSet, Set}, DatabaseConnection, DbErr, EntityTrait};

use crate::{setting::dyn_settings::ActiveModel, upcoming_handler::{blue::TbaMatch, upcoming_game, upcoming_team}};

pub enum UpcomingGameError {
    Database(DbErr),
    Parse(ParseIntError),
}

pub async fn insert_upcoming_game(db: &DatabaseConnection, value: &TbaMatch, event: &String) -> Result<(), UpcomingGameError> {

    let formatted_level: String = match value.comp_level.as_str() {
        "qm" => ("Qualification".to_string()),
        "sf" => ("Playoff".to_string()),
        "f" => ("Playoff/Finals".to_string()),
        "ef" => ("Playoff/Eighth-Finals".to_string()),
        "qf" => ("Playoff/Quarterfinals".to_string()), 
        a=> (a.to_string()), // Fallback
    };

    let game = upcoming_game::ActiveModel {
        id: NotSet,
        event_code: Set(event.clone()),
        match_number: Set(value.match_number),
        set_number: Set(value.set_number),
        tournament_level: Set(formatted_level),
    };

    let id = match upcoming_game::Entity::insert(game).exec(db).await {
        Ok(a) => a,
        Err(a) => {return Err(UpcomingGameError::Database(a));},
    };

    let mut team = Vec::with_capacity(6);

    for (i, name) in value.alliances.blue.team_keys.iter().enumerate() {

        let team_string = if name.chars().last() == Some('B') {
            "99".to_owned() + &name[3..name.len()-1]
        } else {
            (&name[3..]).to_string()
        };
        println!("{:?} {:?}", name , team_string.to_owned());
        let team_number = match team_string.parse() {
            Ok(a) => a,
            Err(a) => {
                return Err(UpcomingGameError::Parse(a));
            }
        };

        team.push(upcoming_team::ActiveModel {
            id: NotSet,
            station: Set(format!("Blue {:?}",i+1)),
            team: Set(team_number),
            scouter: NotSet,
            game_id: Set(id.last_insert_id),
        });
    }

    for (i, name) in value.alliances.red.team_keys.iter().enumerate() {

        let team_string = if name.chars().last() == Some('B') {
            "99".to_owned() + &name[3..name.len()-1]
        } else {
            (&name[3..]).to_string()
        };
        println!("{:?} {:?}", name , team_string.to_owned());
        let team_number = match team_string.parse() {
            Ok(a) => a,
            Err(a) => {
                return Err(UpcomingGameError::Parse(a));
            }
        };

        team.push(upcoming_team::ActiveModel {
            id: NotSet,
            station: Set(format!("Red {:?}",i+1)),
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
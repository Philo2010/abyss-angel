use std::collections::HashMap;

use sea_orm::{ActiveValue::NotSet, ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use sea_orm::Set;
use crate::entity::{pit_upcoming, upcoming_game, upcoming_team};


pub async fn create_pit_upcoming(db: &DatabaseConnection, event_code: &String) -> Result<(), DbErr> {
    let mut teams_hash: HashMap<(i32, bool), ()> = HashMap::new(); 

    let games = upcoming_game::Entity::find()
        .filter(upcoming_game::Column::EventCode.eq(event_code))
        .all(db).await?;

    for game in games {
        let teams = upcoming_team::Entity::find()
            .filter(upcoming_team::Column::GameId.eq(game.id))
            .all(db)
            .await?;

        for team in teams {
            teams_hash.insert((team.team, team.is_ab_team), ());
        }
    }

    let team_vec: Vec<(i32, bool)> = teams_hash.into_keys().collect(); 
    let mut active_pit: Vec<pit_upcoming::ActiveModel> = Vec::new();

    for team_data in team_vec {
        active_pit.push(pit_upcoming::ActiveModel {
            id: NotSet,
            user: Set(None),
            team: Set(team_data.0),
            is_ab_team: Set(team_data.1),
            event_code: Set(event_code.clone()),
            pit_header_id: Set(None),
        });
    }
    pit_upcoming::Entity::insert_many(active_pit).exec(db).await?;
    Ok(())
}
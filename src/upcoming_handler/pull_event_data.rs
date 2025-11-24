use rocket_dyn_templates::{Template, context};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter};

use crate::upcoming_handler::{self, upcoming_team};
use crate::upcoming_handler::upcoming_game;

pub enum EventDataErr {
    Data(DbErr),
    Team(DbErr)

}

pub async fn pull_event_data(db: &DatabaseConnection, event: Option<&str>) -> Result<Vec<(upcoming_game::Model, Vec<upcoming_team::Model>)> , EventDataErr> {
    
    let data;
    if let Some(e) = event {
        data = match upcoming_handler::upcoming_game::Entity::find()
        .filter(upcoming_game::Column::EventCode.contains(e)).all(db).await {
            Ok(a) => a,
            Err(a) => {
                return Err(EventDataErr::Data(a));
            },
        };
    } else {
        data = match upcoming_handler::upcoming_game::Entity::find().all(db).await {
            Ok(a) => a,
            Err(a) => {
                return Err(EventDataErr::Data(a));
            },
        };
    }

    let mut teams: Vec<Vec<upcoming_team::Model>> = Vec::with_capacity(data.len());

    for game in &data {
        teams.push(match game
            .find_related(upcoming_team::Entity)
            .all(db)
            .await {
                Ok(a) => a,
                Err(a) => {
                    return Err(EventDataErr::Team(a));
                },
            });
    }

    Ok(data.into_iter().zip(teams.into_iter()).collect())
}
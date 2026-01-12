use schemars::JsonSchema;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QuerySelect};
use serde::{Deserialize, Serialize};

use crate::entity::users;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SnowScouterDataLeaderBoard {
    pub username: String,
    pub amount_of_warning: i32
}

pub async fn get_snowgrave_leader_board(db: &DatabaseConnection) -> Result<Vec<SnowScouterDataLeaderBoard>, DbErr> {
    let scouters: Vec<(String, i32)> = users::Entity::find()
        .select_only()
        .column(users::Column::Name)
        .column(users::Column::AmountOfWarning)
        .into_tuple()
        .all(db).await?;
    
    let mut res: Vec<SnowScouterDataLeaderBoard> = scouters.into_iter().map(|x| {
        SnowScouterDataLeaderBoard {
            username: x.0,
            amount_of_warning: x.1
        }
    }).collect();

    res.sort_by_key(|x| x.amount_of_warning);


    Ok(res)
}
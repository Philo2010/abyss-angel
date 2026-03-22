use schemars::{JsonSchema};
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{entity::{game_scouts, genertic_header, mvp_data, mvp_scouters, upcoming_game}, snowgrave::datatypes::Team};
use crate::snowgrave::check_system::check;



#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MvpInsert {
    pub mvp_id: i32,
    pub mvp_team: Team,
    pub comment: String,
    pub total_score: i32,
    pub penalty_score: i32,
}




pub async fn insert_mvp_data(data: MvpInsert, db: &DatabaseConnection) -> Result<(), DbErr> {
    //Check if mvp scouter is even real
    let mvp_mod = mvp_scouters::Entity::find_by_id(data.mvp_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find scouter!".to_string()))?;
    if mvp_mod.data.is_some() {
        return Err(DbErr::Custom("There is already data!".to_string()));
    }
    let mut mvp: mvp_scouters::ActiveModel = mvp_mod.clone().into();

    //Insert data into db
    let mvp_insert = mvp_data::ActiveModel {
        id: NotSet,
        mvp_team: Set(data.mvp_team.number),
        mvp_is_ab_team: Set(data.mvp_team.is_ab_team),
        comment: Set(data.comment),
        total_score: Set(data.total_score),
        penalty_score: Set(data.penalty_score),
        is_blue: mvp.is_blue.clone(),
    };
    let data_id = mvp_insert.insert(db).await?;

    mvp.data = Set(Some(data_id.id));

    let is_blue = mvp_mod.is_blue;

    let _res = mvp.update(db).await?;

    let game_id;
    if is_blue {
        game_id = upcoming_game::Entity::find() 
        .filter(upcoming_game::Column::MvpIdBlue.eq(data.mvp_id))  // Return as a tuple (i32,)
        .one(db)
        .await?.ok_or(DbErr::RecordNotFound("Could not find game!".to_string()))?;
    } else {
        game_id = upcoming_game::Entity::find() 
        .filter(upcoming_game::Column::MvpIdRed.eq(data.mvp_id))  // Return as a tuple (i32,)
        .one(db)
        .await?.ok_or(DbErr::RecordNotFound("Could not find game!".to_string()))?;
    }
    
    
    check(game_id.id, db).await?;

    Ok(())
}
use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};

use crate::{entity::{genertic_header, mvp_data, mvp_scouters, upcoming_game}, snowgrave::{check_complete::{self, CheckMatchErr}, datatypes::{MvpData, TeamData}}};




#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MvpInsert {
    pub mvp_id: i32,
    pub mvp_team: TeamData,
    pub comment: String,
    pub total_score: i32,
    pub penalty_score: i32,
}




pub async fn insert_mvp_data(data: MvpInsert, db: &DatabaseConnection) -> Result<(), DbErr> {
    //Check if mvp scouter is even real
    let mvp_mod = mvp_scouters::Entity::find_by_id(data.mvp_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find scouter!".to_string()))?;
    let mut mvp: mvp_scouters::ActiveModel = mvp_mod.clone().into();

    //Insert data into db
    let mvp_insert = mvp_data::ActiveModel {
        id: NotSet,
        mvp_team: Set(data.mvp_team.team),
        mvp_is_ab_team: Set(data.mvp_team.is_ab_team),
        comment: Set(data.comment),
        total_score: Set(data.total_score),
        penalty_score: Set(data.penalty_score),
        is_blue: NotSet,
    };
    let data_id = mvp_insert.update(db).await?;

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
    

    //great now to insert this into the right game

    let match_data = match genertic_header::Entity::find()
        .filter(genertic_header::Column::EventCode.eq(game_id.event_code))
        .filter(genertic_header::Column::MatchId.eq(game_id.match_id))
        .filter(genertic_header::Column::Set.eq(game_id.set))
        .filter(genertic_header::Column::TournamentLevel.eq(game_id.tournament_level))
        .filter(genertic_header::Column::Team.eq(data.mvp_team.team))
        .filter(genertic_header::Column::IsAbTeam.eq(data.mvp_team.is_ab_team))
        .one(db).await? {
            Some(a) => {
                let mut active: genertic_header::ActiveModel = a.into();
                active.is_mvp = Set(true);
                let _res = genertic_header::Entity::update(active).exec(db).await?;
            },
            None => {
                //The data is not here yet, it will be inserted when the scouter inserts there data
            },
        };

    
    let res = match check_complete::check_match(game_id.id, db).await {
        Err(CheckMatchErr::DbErr(a)) => {
            return Err(a);
        }
        _ => {
            ()
        }
    };

    Ok(res)
}
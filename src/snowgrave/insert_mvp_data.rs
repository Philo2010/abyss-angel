use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect};

use crate::{entity::{genertic_header, mvp_data, mvp_scouters, upcoming_game}, snowgrave::{check_complete::{self, CheckMatchErr}, datatypes::{MvpData, TeamData}}};



struct MvpInsert {
    pub mvp_team: TeamData,
    pub comment: String,
    pub total_score_for_red: i32,
    pub total_score_for_blue: i32,
    pub penalty_score_for_red: i32,  //Given by blue
    pub penalty_score_for_blue: i32, //Given by red
}




pub async fn insert_mvp_data(mvp_id: i32, data: MvpInsert, db: &DatabaseConnection) -> Result<(), CheckMatchErr> {
    //Check if mvp scouter is even real
    let mut mvp: mvp_scouters::ActiveModel = mvp_scouters::Entity::find_by_id(mvp_id).one(db).await?.ok_or(DbErr::RecordNotFound("Could not find scouter!".to_string()))?.into();

    //Insert data into db
    let mvp_insert = mvp_data::ActiveModel {
        id: NotSet,
        mvp_team: Set(data.mvp_team.team),
        mvp_is_ab_team: Set(data.mvp_team.is_ab_team),
        comment: Set(data.comment),
        total_score_for_red: Set(data.total_score_for_red),
        total_score_for_blue: Set(data.total_score_for_blue),
        penalty_score_for_red: Set(data.penalty_score_for_red),
        penalty_score_for_blue: Set(data.penalty_score_for_blue),
    };
    let data_id = mvp_data::Entity::insert(mvp_insert).exec(db).await?.last_insert_id;

    mvp.data = Set(Some(data_id));


    let _res = mvp.update(db).await?;

    let game_id = upcoming_game::Entity::find() 
        .filter(upcoming_game::Column::MvpId.eq(mvp_id))  // Return as a tuple (i32,)
        .one(db)
        .await?.ok_or(DbErr::RecordNotFound("Could not find game!".to_string()))?;

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

    
    let res = check_complete::check_match(game_id.id, db).await?;

    Ok(res)
}
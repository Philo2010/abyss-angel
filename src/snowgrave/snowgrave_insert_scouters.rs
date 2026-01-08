use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, Database, DatabaseConnection, DbErr, EntityTrait};
use uuid::Uuid;

use crate::{backenddb::example_game::ActiveModel, entity::{game_scouts, mvp_scouters, sea_orm_active_enums::Stations, upcoming_game, upcoming_team}};


struct ScouterInsertForm {
    player_indexs: Vec<Uuid>,
    //id is a ref to a team, Uuid is the MVP scouter
    matches: Vec<(i32, Vec<GameTeamDataScouter>, Uuid)>,
}
struct GameTeamDataScouter {
    id: usize,
    station: Stations, //For mvp, this will be ignored just fill in junk
}


pub async fn insert_scouters(form: ScouterInsertForm, db: &DatabaseConnection) -> Result<(), DbErr> {
    
    let mut scouters: Vec<game_scouts::ActiveModel> = Vec::with_capacity(form.matches.len()); //a bit smaller but eh
    let mut teams: HashMap<i32, upcoming_team::Model> = HashMap::new();
    
    for matche in form.matches {
        let team = if let Some(team) = teams.get(&matche.0) {
            team
        } else {
            let model = upcoming_team::Entity::find_by_id(matche.0)
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("No team found".to_string()))?;

            teams.insert(matche.0, model);
            teams.get(&matche.0).unwrap()
        };

        //create mvp scouter
        let mvp: mvp_scouters::ActiveModel = mvp_scouters::ActiveModel { 
            id: NotSet, 
            scouter: Set(matche.2), 
            data: Set(None) };
        let mvp_id = mvp.insert(db).await?.id;
        let game_data = upcoming_game::Entity::find_by_id(team.game_id).one(db).await?.ok_or(DbErr::Custom("could not find game".to_string()))?;
        let mut game_active: upcoming_game::ActiveModel = game_data.into();
        game_active.mvp_id = Set(Some(mvp_id));
        game_active.update(db).await?;

        for scouter in matche.1 {
            let scouter_id = form.player_indexs.get(scouter.id).ok_or(DbErr::Custom("Invaild Index for players!".to_string()))?;
            
            let scouter = game_scouts::ActiveModel {
                id: NotSet,
                game_id: Set(team.game_id),
                team_id: Set(team.id),
                scouter_id: Set(*scouter_id),
                done: Set(false), //always
                station: sea_orm::Set(scouter.station),
                is_redo: Set(false)
            };
            scouters.push(scouter);
        }

    }

    game_scouts::Entity::insert_many(scouters).exec(db).await?;

    Ok(())
}
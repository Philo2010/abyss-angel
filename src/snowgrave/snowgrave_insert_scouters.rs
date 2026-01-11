use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, Database, DatabaseConnection, DbErr, EntityTrait};
use uuid::Uuid;

use crate::{backenddb::example_game::ActiveModel, entity::{game_scouts, mvp_scouters, sea_orm_active_enums::Stations, upcoming_game, upcoming_team}};


struct ScouterInsertForm {
    player_indexs: Vec<Uuid>,
    //id is a ref to a team, Uuid is the MVP scouter
    matches: Vec<(i32, Vec<GameTeamDataScouter>, GameTeamDataMvp)>,
}
struct GameTeamDataScouter {
    id: usize,
    station: Stations, 
}
struct GameTeamDataMvp {
    red: Uuid,
    blue: Uuid
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

        //create mvp scouters
        let mvp_red: mvp_scouters::ActiveModel = mvp_scouters::ActiveModel { 
            id: NotSet, 
            scouter: Set(matche.2.red), 
            is_blue: Set(false),
            data: Set(None)
        };
        let mvp_id_red = mvp_red.insert(db).await?.id;

        let mvp_blue: mvp_scouters::ActiveModel = mvp_scouters::ActiveModel { 
            id: NotSet, 
            scouter: Set(matche.2.blue), 
            is_blue: Set(true),
            data: Set(None)
        };
        let mvp_id_blue = mvp_blue.insert(db).await?.id;

        let game_data = upcoming_game::Entity::find_by_id(team.game_id).one(db).await?.ok_or(DbErr::Custom("could not find game".to_string()))?;
        let mut game_active: upcoming_game::ActiveModel = game_data.into();
        game_active.mvp_id_red = Set(Some(mvp_id_red));
        game_active.mvp_id_blue = Set(Some(mvp_id_blue));
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
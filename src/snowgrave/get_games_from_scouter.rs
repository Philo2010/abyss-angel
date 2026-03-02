use std::collections::HashMap;

use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter
};
use uuid::Uuid;

use crate::{
    entity::{
        game_scouts, mvp_scouters, sea_orm_active_enums::TournamentLevels, upcoming_game, upcoming_team
    }, snowgrave::{datatypes::{Game, MvpPair, MvpUpcoming, Team}, db_models_to_snow::{get_mvp, get_mvps}},
};

pub struct ScouterField {
    pub event_code: String,
    pub match_id: i32,
    pub set: i32,
    pub tournament_level: TournamentLevels,
    pub teams: Team,
    pub mvp: MvpPairScouterField,
    pub is_redo: bool
}


pub struct MvpPairScouterField {
    pub red: Option<MvpUpcoming>,
    pub blue:  Option<MvpUpcoming>
}

pub async fn get_games_for_scouter(
    scouter: Uuid,
    db: &DatabaseConnection,
) -> Result<Vec<Game>, DbErr> {
    //get all the scouting entrys first
    let scouter_entrys = game_scouts::Entity::find()
        .filter(game_scouts::Column::ScouterId.eq(scouter))
        .all(db).await?;
    let mut scout_final: Vec<ScouterField> = Vec::new();

    let mut game_cache: HashMap<i32, upcoming_game::Model> = HashMap::new();
    let mut team_cache: HashMap<i32, upcoming_team::Model> = HashMap::new();
    
    for entry in &scouter_entrys {

        let game = match game_cache.get(&entry.game_id) {
            Some(a) => {
                a
            },
            None => {
                let game_model = match upcoming_game::Entity::find_by_id(entry.game_id).one(db).await? {
                    Some(a) => {
                        a
                    },
                    None => {
                        panic!("THIS STATE SHOULD NOT HAPPEN, GAME ID SHOULD ALWAYS BE VALID");
                    },
                };
                game_cache.insert(entry.game_id, game_model);
                game_cache.get(&entry.game_id).unwrap()
            },
        };

        let team = match team_cache.get(&entry.team_id) {
            Some(a) => {
                a
            },
            None => {
                let team_model = match upcoming_team::Entity::find_by_id(entry.team_id).one(db).await? {
                    Some(a) => {
                        a
                    },
                    None => {
                        panic!("THIS STATE SHOULD NOT HAPPEN, TEAM ID SHOULD ALWAYS BE VALID");
                    },
                };
                team_cache.insert(entry.team_id, team_model);
                team_cache.get(&entry.team_id).unwrap()
            },
        };


        let mvp_red: Option<MvpUpcoming>;
        if let Some(mvp_red_id) = game.mvp_id_red {
            mvp_red = Some(get_mvp(mvp_red_id, db).await?);
        } else {
            mvp_red = None;
        }

        let mvp_blue: Option<MvpUpcoming>;
        if let Some(mvp_blue_id) = game.mvp_id_blue {
            mvp_blue = Some(get_mvp(mvp_blue_id, db).await?);
        } else {
            mvp_blue = None;
        }
        let mvps = MvpPairScouterField {
            red: mvp_red,
            blue: mvp_blue
        };

        scout_final.push(ScouterField {
            event_code: game.event_code.clone(),
            match_id: game.match_id,
            set: game.set,
            tournament_level: game.tournament_level,
            teams: Team { 
                number: team.team,
                is_ab_team: team.is_ab_team
            },
            mvp: mvps,
            is_redo: entry.is_redo
        });
    }


    todo!()
}

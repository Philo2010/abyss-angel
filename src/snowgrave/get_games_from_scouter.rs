use std::collections::HashMap;

use schemars::JsonSchema;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entity::{
        game_scouts, mvp_scouters, sea_orm_active_enums::TournamentLevels, upcoming_game, upcoming_team
    }, snowgrave::{datatypes::{Game, GamePart, MvpPair, MvpPairPart, MvpUpcoming, ScoutGame, ScoutMatch, Team}, db_models_to_snow::{get_mvp, get_mvps}},
};

//Somtims, dipit what that muh sam utlaman says, the clanker can do suth
/*
⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⡀⠀⢀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡞⠁⡇⡼⢉⢌⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣣⢀⣇⢁⡏⡞⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⣤⡤⠴⢶⣚⣛⠉⣥⣤⣤⡈⠈⠼⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣴⣶⣶⣾⣿⣏⠉⠁⠀⢀⠠⠈⡳⣿⠀⢻⢶⣷⠁⠀⠀⢻⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⣿⣿⣿⣿⣿⣿⠀⠀⠈⠀⠀⠀⠈⠀⠀⠀⠈⠀⠀⠀⢀⢸⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠰⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢹⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠐⡀⠆⡀⢹⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠿⣿⣿⡿⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣰⠦⠅⠂⠐⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠲⠤⢤⢤⢤⠀⠀⠀⠐⢒⣛⣭⠁⠂⠀⠠⠈⣸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣆⣀⠀⠀⠈⠉⠀⢀⣀⣰⣶⡿⠿⢶⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣞⠈⠙⠛⠯⠯⠭⠯⠽⠓⠛⠉⣀⡀⠀⠣⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡘⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢏⠀⠀⠀⢣⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢃⢹⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡸⠢⠌⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢄⡷⣤⣤⣀⣄⣀⣀⣀⣀⣀⣤⣅⠀⠀⢀⡜⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢧⠀⠹⡝⣺⡷⡗⠛⠉⠉⠀⠙⡳⢲⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⡀⠀⠀⠉⢙⡏⠉⠉⠀⠀⠈⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠇⠠⣠⡀⠀⢲⠀⢀⡀⠀⠀⠀⠘⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠼⣀⠀⠀⠀⠘⣆⠀⠈⠂⠀⠀⠀⣇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⡀⠈⠉⠉⠁⢰⠉⠙⠓⠒⠒⠉⢽⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠒⠉⠙⢻⣹⡿⢶⡖⣦⣶⣶⣶⣶⡖⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡀⠐⠉⠀⢱⢫⣶⠋⠁⢈⠙⢿⠷⣹⡞⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⠛⠛⠓⠉⠉⢖⣀⡀⠀⢐⣋⠝⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣉⣉⣉⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀
 */

pub async fn get_games_for_scouter(
    scouter: Uuid,
    db: &DatabaseConnection,
) -> Result<Vec<GamePart>, DbErr> {
    //get all the scouting entrys first
    let scouter_entrys = game_scouts::Entity::find()
        .filter(game_scouts::Column::ScouterId.eq(scouter))
        .all(db).await?;

    let mut game_cache: HashMap<i32, GamePart> = HashMap::new();
    let mut team_cache: HashMap<i32, upcoming_team::Model> = HashMap::new();
    let mut mvp_cache: HashMap<i32, MvpPair> = HashMap::new(); //the i32 is the game id

    //create the all mightly cashe to be used
    for entry in &scouter_entrys {
        //cache the game
        if !game_cache.contains_key(&entry.game_id) {
            let game = upcoming_game::Entity::find_by_id(entry.game_id).one(db).await?
                .ok_or(DbErr::Custom("Invaild Game Id (should never happen)".to_string()))?;
            //grab mvp
            let mut mvp: MvpPairPart = MvpPairPart { 
                red: None, 
                blue: None
            };

            if let Some(mvp_red_id) = game.mvp_id_red {
                let model_red = get_mvp(mvp_red_id, db).await?;
                mvp.red = Some(model_red);
            }

            if let Some(mvp_blue_id) = game.mvp_id_blue {
                let model_blue = get_mvp(mvp_blue_id, db).await?;
                mvp.blue = Some(model_blue);
            }

            let game_better = GamePart {
                event_code: game.event_code,
                match_id: game.match_id,
                set: game.set,
                tournament_level: game.tournament_level,
                scout: ScoutGame {
                    red_1: Vec::new(),
                    red_2:  Vec::new(),
                    red_3:  Vec::new(),
                    blue_1:  Vec::new(),
                    blue_2:  Vec::new(),
                    blue_3:  Vec::new()
                },
                mvp,
            };


            game_cache.insert(game.id,game_better);
        }

        //cache the team
        if !team_cache.contains_key(&entry.team_id) {
            let team = upcoming_team::Entity::find_by_id(entry.team_id).one(db).await?
                .ok_or(DbErr::Custom("Invaild Game Id (should never happen)".to_string()))?;
            team_cache.insert(team.id,team);
        }
    }

    //Since im stupid, the strutors link down to up, do i care? yes? but like most of my problem i will make this work and go back to vulkan my beloved
    //make the master game objects first for linking

    //time to loop the lowest element and create our tree!
    for scouter in scouter_entrys {
        //first game team info
        let team = team_cache.get(&scouter.team_id).unwrap(); //safe as we know it would be there

        let scout_match = ScoutMatch {
            id: scouter.id,
            name: scouter.scouter_id,
            station: scouter.station,
            done: scouter.done,
            is_redo: scouter.is_redo,
            team: Team {
                number: team.team,
                is_ab_team: team.is_ab_team
            },
            data: None, //this is the method we use to get data (even in edit we basicly submit a new entry that only keeps the same ID)
        };

        match scout_match.station {
            crate::entity::sea_orm_active_enums::Stations::Red1 => {
                game_cache.get_mut(&scouter.game_id).unwrap().scout.red_1.push(scout_match);
            },
            crate::entity::sea_orm_active_enums::Stations::Red2 => {
                game_cache.get_mut(&scouter.game_id).unwrap().scout.red_2.push(scout_match);
            },
            crate::entity::sea_orm_active_enums::Stations::Red3 => {
                game_cache.get_mut(&scouter.game_id).unwrap().scout.red_3.push(scout_match);
            },
            crate::entity::sea_orm_active_enums::Stations::Blue1 => {
                game_cache.get_mut(&scouter.game_id).unwrap().scout.blue_1.push(scout_match);
            },
            crate::entity::sea_orm_active_enums::Stations::Blue2 => {
                game_cache.get_mut(&scouter.game_id).unwrap().scout.blue_2.push(scout_match);
            },
            crate::entity::sea_orm_active_enums::Stations::Blue3 => {
                game_cache.get_mut(&scouter.game_id).unwrap().scout.blue_3.push(scout_match);
            },
        }
    }



    return Ok(game_cache.into_iter().map(|x|x.1).collect());
    
}

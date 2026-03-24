use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::{SETTINGS, backenddb::game::game_dispatch, entity::{game_scouts, mvp_data, mvp_scouters, scout_game_midway_insert, upcoming_game, upcoming_team}, frontend::snowgrave::get_all_scouts::Game, snowgrave::datatypes::{self, MvpData, MvpPair, MvpUpcoming, ScoutGame, ScoutMatch, ScoutMatchData, Team}};



//Id should be from mvp_scouters
pub async fn get_mvp(id: i32, db: &DatabaseConnection) -> Result<MvpUpcoming, DbErr> {
    let a = mvp_scouters::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::Custom(format!("Missing scouter id {}", id)))?;

    let data = match a.data {
        Some(e) => {
            let z = mvp_data::Entity::find_by_id(e)
                .one(db)
                .await?
                .ok_or_else(|| DbErr::Custom(format!("Invalid mvp_data id {}", e)))?;

            Some(MvpData {
                mvp_team: Team {
                    number: z.mvp_team,
                    is_ab_team: z.mvp_is_ab_team,
                },
                comment: z.comment,
                is_blue: z.is_blue,
                total_score: z.total_score,
                penalty_score: z.penalty_score,
            })
        }
        None => None,
    };

    Ok(MvpUpcoming {
        id: a.id,
        name: a.scouter,
        data,
    })
}

pub async fn get_mvps(id_red: i32, id_blue: i32, db: &DatabaseConnection) -> Result<MvpPair, DbErr> {
    Ok(MvpPair {
        red: get_mvp(id_red, db).await?,
        blue: get_mvp(id_blue, db).await?,
    })
}

pub async fn get_scouts(upcoming_team_id: i32, db: &DatabaseConnection) -> Result<Vec<ScoutMatch>, DbErr> {
    let upcoming_team = match upcoming_team::Entity::find_by_id(upcoming_team_id).one(db).await? {
        Some(a) => {
            a
        },
        None => {
            return Err(DbErr::Custom("Could not find the upcoming team id".to_string()));
        },
    };

    let scouts = game_scouts::Entity::find()
        .filter(game_scouts::Column::TeamId.eq(upcoming_team.id))
        .all(db).await?;

    //check edgecases
    if scouts.is_empty() {
        return Err(DbErr::Custom("Scouts have not been selected yet!".to_string()));
    }

    if scouts.len() >= 6 {
        return Err(DbErr::Custom("Expected at least 6 scouts".to_owned())); //a bit weird
    }

    let mut scouts_nice: Vec<ScoutMatch> = Vec::new();
    

    for scout in scouts {
        let midpoint: Option<ScoutMatchData>;
        if let Some(mid) = scout.game_midway {
            let mid_data = match scout_game_midway_insert::Entity::find_by_id(mid).one(db).await? {
                Some(a) => a,
                None => {
                    return Err(DbErr::Custom("Mid data value is invalid (should not happen normaly)".to_string()));
                },
            };

            let functions = game_dispatch(mid_data.game_type_id);
            let game = functions.get(mid_data.game_id, db).await?;

            midpoint = Some(ScoutMatchData {
                match_id: mid_data.match_id,
                set: mid_data.set,
                total_score: mid_data.total_score,
                teleop_score: mid_data.teleop_score,
                auto_score: mid_data.auto_score,
                defence: mid_data.defence,
                comment: mid_data.comment,
                event_code: mid_data.event_code, 
                tournament_level: mid_data.tournament_level,
                station: mid_data.station,
                created_at: mid_data.created_at,
                game: game,
                game_id: mid_data.game_id,
                game_type_id: mid_data.game_type_id,
            });
        } else {
            midpoint = None
        }
    
        scouts_nice.push(ScoutMatch {
            name: scout.scouter_id,
            station: scout.station,
            done: scout.done,
            is_redo: scout.is_redo,
            team: Team {
                number: upcoming_team.team,
                is_ab_team: upcoming_team.is_ab_team,
            },
            data: midpoint,
            id: scout.id,
        });
    }


    Ok(scouts_nice)
}


struct ScoutGameButNotCool { 
    pub red_1: Option<Vec<ScoutMatch>>,
    pub red_2: Option<Vec<ScoutMatch>>,
    pub red_3: Option<Vec<ScoutMatch>>,
    pub blue_1: Option<Vec<ScoutMatch>>,
    pub blue_2: Option<Vec<ScoutMatch>>,
    pub blue_3: Option<Vec<ScoutMatch>>,
}

impl ScoutGameButNotCool {
    pub fn into_cool(self) -> Option<ScoutGame> {
        match self {
            ScoutGameButNotCool {
                red_1: Some(red_1),
                red_2: Some(red_2),
                red_3: Some(red_3),
                blue_1: Some(blue_1),
                blue_2: Some(blue_2),
                blue_3: Some(blue_3),
            } => Some(ScoutGame {
                red_1,
                red_2,
                red_3,
                blue_1,
                blue_2,
                blue_3,
            }),
            _ => None,
        }
    }
}

/// Expects all scouters to be selected at this point or well will not return anything!
//id is a upcoming-game id
pub async fn get_game(id: i32, db: &DatabaseConnection) -> Result<datatypes::Game, DbErr> {
    //get the game object from db, this will be how we fetch the rest
    let game_prim = match upcoming_game::Entity::find_by_id(id).one(db).await? {
        Some(a) => a,
        None => {
            return Err(DbErr::Custom("[GET_GAME] Invaild id! (game)".to_string()));
        },
    };

    //get the mvps (else they shall be null)
    let mvps: MvpPair;
    if let Some(mvp_id_red) = game_prim.mvp_id_red {
        if let Some(mvp_id_blue) = game_prim.mvp_id_blue {
            mvps = get_mvps(mvp_id_red, mvp_id_blue, db).await?;
        } else {
            return Err(DbErr::Custom("Blue MVP is not been selected yet!".to_string()));
        }
    } else {
        return Err(DbErr::Custom("Blue MVP is not been selected yet!".to_string()));
    }

    //get all the teams
    let teams = upcoming_team::Entity::find()
        .filter(upcoming_team::Column::GameId.eq(game_prim.id))
        .all(db).await?;
    
    assert!(teams.len() == 6, "INVAID AMOUNT FOR TEAMS");
    let mut scout_info: ScoutGameButNotCool = ScoutGameButNotCool {
        red_1: None,
        red_2: None,
        red_3: None,
        blue_1: None,
        blue_2: None,
        blue_3: None,
    };

    for team in teams {
        match team.station {
            crate::entity::sea_orm_active_enums::Stations::Red1 => {
                scout_info.red_1 = Some(get_scouts(team.id, db).await?);
            },
            crate::entity::sea_orm_active_enums::Stations::Red2 => {
                scout_info.red_2 = Some(get_scouts(team.id, db).await?);
            },
            crate::entity::sea_orm_active_enums::Stations::Red3 => {
                scout_info.red_3 = Some(get_scouts(team.id, db).await?);
            },
            crate::entity::sea_orm_active_enums::Stations::Blue1 => {
                scout_info.blue_1 = Some(get_scouts(team.id, db).await?);
            },
            crate::entity::sea_orm_active_enums::Stations::Blue2 => {
                scout_info.blue_2 = Some(get_scouts(team.id, db).await?);
            },
            crate::entity::sea_orm_active_enums::Stations::Blue3 => {
                scout_info.blue_3 = Some(get_scouts(team.id, db).await?);
            },
        }
    }

    //check any none
    let scout_info_cool = match scout_info.into_cool() {
        Some(a) => {
            a
        },
        None => {
            panic!("INVAILD STATE IN STATIONS");
        },
    };
    

    Ok(datatypes::Game {
        event_code: game_prim.event_code,
        match_id: game_prim.match_id,
        set: game_prim.set,
        tournament_level: game_prim.tournament_level,
        scout: scout_info_cool,
        mvp: mvps,
    })
}
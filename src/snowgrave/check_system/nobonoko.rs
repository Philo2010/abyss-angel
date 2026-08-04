//this file defineds a endpoint to allow for a force submit despite Max's orders
//I have done this because no, this is really useful, why in hell would i ever get rid of this

//!! This is a rebuilt spetfic moduale, it should not be however and should be genetric but beca
//! /use of time constrates h ave not made it like that, so change it in the FUTURE!!!!
//!TODO, make this a major type

use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, prelude::Expr};

use crate::{auth::get_by_user::get_by_uuid, backenddb::{entrys::rebuilt::Insert, game::{GamesInserts, GamesInsertsSpecific, HeaderInsert}}, entity::{game_scouts, sea_orm_active_enums::Stations}, snowgrave::{check_system::db_work::publish, datatypes::{Game, ScoutMatch, Team}, db_models_to_snow}};
pub enum Alliance {
    Red,
    Blue
}

async fn get_scout(scouts: &[ScoutMatch], label: &str, db: &DatabaseConnection) -> Result<ScoutMatch, DbErr> {
    let x = scouts.first().ok_or_else(|| {
        DbErr::Custom(format!("No scout found for {label}"))
    })?;

    if x.data.is_none() {
        let user = get_by_uuid(&x.name, db).await.unwrap();
        return Err(DbErr::Custom(format!(
            "User {user} has not completed their {label} scout, can't override!"
        )));
    }

    Ok(x.clone())
}


async fn check_data_present(part_model: Game, alliance: Alliance, db: &DatabaseConnection) -> Result<([ScoutMatch; 3], Team) ,DbErr> {
    match alliance {
        Alliance::Red => {
            //now we check that at least, there is data, we dont care if its done however, since this is a overide of the checking system
            let red_1 = get_scout(&part_model.scout.red_1, "Red 1", db).await?;
            let red_2 = get_scout(&part_model.scout.red_2, "Red 2", db).await?;
            let red_3 = get_scout(&part_model.scout.red_3, "Red 3", db).await?;
            let team;
            if let Some(mvp) = part_model.mvp.red.data {
                team = mvp.mvp_team;
            } else {
                return Err(DbErr::Custom("Red MVP is not done yet!".to_string()));
            }
            return Ok(([red_1, red_2, red_3], team))
        }
        Alliance::Blue => {
            //TODO! Allow this to calulate averages, comp is in one day so can't really make this proper
            let blue_1 = get_scout(&part_model.scout.blue_1, "Blue 1", db).await?;
            let blue_2 = get_scout(&part_model.scout.blue_2, "Blue 2", db).await?;
            let blue_3 = get_scout(&part_model.scout.blue_3, "Blue 3", db).await?;
            let team;
            if let Some(mvp) = part_model.mvp.blue.data {
                team = mvp.mvp_team;
            } else {
                return Err(DbErr::Custom("Red MVP is not done yet!".to_string()));
            }
            return Ok(([blue_1, blue_2, blue_3], team))
        },
    }
}


pub async fn bypass_check(game_id: i32, alliance: Alliance, db: &DatabaseConnection) -> Result<(), DbErr> {
    let stations = match &alliance {
        Alliance::Red => vec![Stations::Red1, Stations::Red2, Stations::Red3],
        Alliance::Blue => vec![Stations::Blue1, Stations::Blue2, Stations::Blue3],
    };

    //first we need to grab the parisal data
    let part_model = db_models_to_snow::get_game(game_id, db).await?;
    let (scouts, team) = check_data_present(part_model, alliance, db).await?;
    //no need for warnings or anything else just insert them baby!
    let insert_scouts: Vec<GamesInserts> = scouts.into_iter().map(|x| {
        let is_mvp = x.team == team;
        let game_data = match &x.data.as_ref().unwrap().game {
            crate::backenddb::game::GamesFullSpecific::RebuiltGame(x) => {
                GamesInsertsSpecific::RebuiltGame(Insert {
                    defence_main: x.defence_main,
                    fuel_shoot_teleop: x.fuel_shoot_teleop,
                    fuel_pass_teleop: x.fuel_pass_teleop,
                    fuel_shoot_auto: x.fuel_shoot_auto,
                    fuel_pass_auto: x.fuel_pass_auto,
                    climb_end: x.climb_end,
                    climb_auto: x.climb_auto,
                    beach_on_bump: x.beach_on_bump,
                    dead: x.dead,
                    dnf: x.dnf,
                    auto_time: x.auto_time,
                    dpdg: None,
                })
            }
        };
        GamesInserts {
            header: HeaderInsert {
                user: vec![x.name],
                team: x.team.number,
                is_ab_team: x.team.is_ab_team,
                match_id: x.data.as_ref().unwrap().match_id, //unwrap is safe becasue of check_data_present
                set: x.data.as_ref().unwrap().set,
                defence: x.data.as_ref().unwrap().defence as f32,
                event_code: x.data.as_ref().unwrap().event_code.clone(),
                tournament_level: x.data.as_ref().unwrap().tournament_level,
                station: x.data.as_ref().unwrap().station,
                is_mvp,
                comment: x.data.unwrap().comment,
            },
            game: game_data,
        }
    }).collect();

    publish(insert_scouts, db).await?;

    game_scouts::Entity::update_many()
        .col_expr(game_scouts::Column::Done, Expr::value(true))
        .col_expr(game_scouts::Column::IsRedo, Expr::value(false))
        .filter(game_scouts::Column::GameId.eq(game_id))
        .filter(game_scouts::Column::Station.is_in(stations))
        .exec(db)
        .await?;

    Ok(())
}

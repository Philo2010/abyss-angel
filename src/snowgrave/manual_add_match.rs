//should be used to add a match (mostly used for playoffs in case blue is shiting itself)

use sea_orm::{ActiveValue::{NotSet, Set}, DatabaseConnection, DbErr, EntityTrait, ActiveModelTrait, TransactionError, TransactionTrait};
use uuid::Uuid;

use crate::{entity::{game_scouts, mvp_scouters, sea_orm_active_enums::{Stations, TournamentLevels}, upcoming_game, upcoming_team}, snowgrave::datatypes::Team};


pub struct ManualInsertMatch {
    pub event_code: String,
    pub match_id: i32,
    pub set: i32,
    pub tournament_level: TournamentLevels,
    pub mvps: MvpInsertMatch,
    pub teams_red: [ManualInsertTeam; 3],
    pub teams_blue: [ManualInsertTeam; 3]
}

pub struct MvpInsertMatch {
    pub red_username: Uuid,
    pub blue_username: Uuid
}


pub struct ManualInsertTeam {
    pub team: Team,
    pub scouters: Vec<Uuid>
}

pub async fn manual_add_match(match_insert: ManualInsertMatch, db: &DatabaseConnection) -> Result<(), DbErr> {
    //construct the mvps
    db.transaction::<_, (), DbErr>(|txn| {
    Box::pin(async move {
        //insert MVPs first
        let mvp_red_id = mvp_scouters::Entity::insert(mvp_scouters::ActiveModel {
            id: NotSet,
            scouter: Set(match_insert.mvps.red_username),
            is_blue: Set(false),
            data: Set(None),
        }).exec(txn).await?.last_insert_id;
        let mvp_blue_id = mvp_scouters::Entity::insert(mvp_scouters::ActiveModel {
            id: NotSet,
            scouter: Set(match_insert.mvps.blue_username),
            is_blue: Set(true),
            data: Set(None),
        }).exec(txn).await?.last_insert_id;

        let game_id = upcoming_game::Entity::insert(upcoming_game::ActiveModel {
            id: NotSet,
            event_code: Set(match_insert.event_code),
            match_id: Set(match_insert.match_id),
            set: Set(match_insert.set),
            mvp_id_blue: Set(Some(mvp_blue_id)),
            mvp_id_red: Set(Some(mvp_red_id)),
            tournament_level: Set(match_insert.tournament_level),
        }).exec(txn).await?.last_insert_id;

        for (index, team) in match_insert.teams_red.iter().enumerate() {
            let station = match index {
                0 => Stations::Red1,
                1 => Stations::Red2,
                2 => Stations::Red3,
                _ => unreachable!()
            };

            
            let team_id = upcoming_team::Entity::insert(upcoming_team::ActiveModel {
                id: NotSet,
                station: Set(station),
                is_ab_team: Set(team.team.is_ab_team),
                team: Set(team.team.number),
                game_id: Set(game_id),
            }).exec(txn).await?.last_insert_id;

            for scouter in &team.scouters {
                let scouter_id = game_scouts::Entity::insert(game_scouts::ActiveModel {
                    id: NotSet,
                    game_id: Set(game_id),
                    team_id: Set(team_id),
                    scouter_id: Set(*scouter),
                    station: Set(station),
                    game_midway: Set(None),
                    done: Set(false),
                    is_redo: Set(false),
                }).exec(txn).await?.last_insert_id;
            }
            
        }

        for (index, team) in match_insert.teams_blue.iter().enumerate() {
            let station = match index {
                0 => Stations::Blue1,
                1 => Stations::Blue2,
                2 => Stations::Blue3,
                _ => unreachable!()
            };

            
            let team_id = upcoming_team::Entity::insert(upcoming_team::ActiveModel {
                id: NotSet,
                station: Set(station),
                is_ab_team: Set(team.team.is_ab_team),
                team: Set(team.team.number),
                game_id: Set(game_id),
            }).exec(txn).await?.last_insert_id;

            for scouter in &team.scouters {
                let scouter_id = game_scouts::Entity::insert(game_scouts::ActiveModel {
                    id: NotSet,
                    game_id: Set(game_id),
                    team_id: Set(team_id),
                    scouter_id: Set(*scouter),
                    station: Set(station),
                    game_midway: Set(None),
                    done: Set(false),
                    is_redo: Set(false),
                }).exec(txn).await?.last_insert_id;
            }
            
        }

        Ok(())
    })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(e) => e,
        TransactionError::Transaction(e) => e,
    })
}
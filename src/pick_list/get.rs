use rocket::State;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use rocket_okapi::JsonSchema;
use serde::Serialize;

use crate::{backenddb::game::{TeamAvg, average_game}, entity::{pick_list, types::Team}, setting::setevent::get_event_inner};



#[derive(Serialize, JsonSchema)]
pub struct PickEntry {
    pub team: Team,
    pub team_avg: TeamAvg,
    pub is_selected_defence: bool,
    pub is_selected_offence: bool,
    pub is_selected_general: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct PickLists {
    pub results: Vec<PickEntry>
}

pub async fn get(db: &DatabaseConnection, include_midway: bool) -> Result<PickLists, DbErr> {
    let current_event = get_event_inner(db).await?;



    let averages = average_game(&current_event, include_midway, db).await?;
    let mut pick_entries: Vec<PickEntry> = Vec::new();

    for average in averages {
        let team = Team {
            number: average.team,
            is_ab_team: average.is_ab_team,
        };
        let pick_entry = pick_list::Entity::find()
            .filter(pick_list::Column::Team.eq(team.number))
            .filter(pick_list::Column::TeamIsAbTeam.eq(team.is_ab_team))
            .filter(pick_list::Column::EventCode.eq(current_event.clone())).one(db).await?;
        let pick_entry_final;
        match pick_entry {
            Some(a) => {pick_entry_final = a},
            None => {
                //need to insert a BRAND new one
                let insert_thingy = pick_list::ActiveModel {
                    id: sea_orm::NotSet,
                    team: sea_orm::Set(team.number),
                    team_is_ab_team: sea_orm::Set(team.is_ab_team),
                    event_code: sea_orm::Set(current_event.clone()),
                    is_selected_defence: sea_orm::Set(false), // by default false
                    is_selected_offence: sea_orm::Set(false),
                    is_selected_general: sea_orm::Set(false),
                };
                let model = pick_list::Entity::insert(insert_thingy).exec_with_returning(db).await?;
                pick_entry_final = model;
            },
        }
    
        pick_entries.push(PickEntry {
            team,
            team_avg: average,
            is_selected_defence: pick_entry_final.is_selected_defence,
            is_selected_offence: pick_entry_final.is_selected_offence,
            is_selected_general: pick_entry_final.is_selected_general 
        });
    }

    Ok(PickLists { results: pick_entries })
}
use std::collections::HashMap;

use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use rocket_okapi::JsonSchema;
use serde::Serialize;
use crate::{backenddb::game::{TeamAvg, average_game}, entity::{pick_list, types::Team}, setting::setevent::get_event_inner, snowgrave::blue::{get_ranking_from_blue, EventRankingNice}};

#[derive(Serialize, JsonSchema)]
pub struct PickEntry {
    pub team: Team,
    pub team_avg: TeamAvg,
    pub is_selected_defence: bool,
    pub is_selected_offence: bool,
    pub is_selected_general: bool,
    pub order_defence: i32,
    pub order_offence: i32,
    pub order_general: i32,
}

#[derive(Serialize, JsonSchema)]
pub struct PickLists {
    pub results: Vec<PickEntry>
}

pub async fn get(db: &DatabaseConnection, include_midway: bool, client: &reqwest::Client) -> Result<PickLists, DbErr> {
    let current_event = get_event_inner(db).await?;

    let tba_rankings = get_ranking_from_blue(client, &current_event).await.unwrap_or_else(|_| {
        EventRankingNice { rankings: Vec::new() }
    });
    let rank_map: HashMap<(i32, bool), i32> = tba_rankings.rankings.iter()
        .map(|r| ((r.team_key.number, r.team_key.is_ab_team), r.rank))
        .collect();

    let default_rank = tba_rankings.rankings.len() as i32 + 1;

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
                let tba_rank = rank_map.get(&(team.number, team.is_ab_team)).copied().unwrap_or(default_rank);
                let insert_thingy = pick_list::ActiveModel {
                    id: sea_orm::NotSet,
                    team: sea_orm::Set(team.number),
                    team_is_ab_team: sea_orm::Set(team.is_ab_team),
                    event_code: sea_orm::Set(current_event.clone()),
                    is_selected_defence: sea_orm::Set(false),
                    is_selected_offence: sea_orm::Set(false),
                    is_selected_general: sea_orm::Set(false),
                    order_defence: sea_orm::Set(Some(tba_rank)),
                    order_offence: sea_orm::Set(Some(tba_rank)),
                    order_general: sea_orm::Set(Some(tba_rank)),
                };
                let model = pick_list::Entity::insert(insert_thingy).exec_with_returning(db).await?;
                pick_entry_final = model;
            },
        }

        let resolve_order = |db_order: Option<i32>| -> i32 {
            db_order.unwrap_or_else(|| {
                rank_map.get(&(team.number, team.is_ab_team)).copied().unwrap_or(default_rank)
            })
        };

        pick_entries.push(PickEntry {
            team,
            team_avg: average,
            is_selected_defence: pick_entry_final.is_selected_defence,
            is_selected_offence: pick_entry_final.is_selected_offence,
            is_selected_general: pick_entry_final.is_selected_general,
            order_defence: resolve_order(pick_entry_final.order_defence),
            order_offence: resolve_order(pick_entry_final.order_offence),
            order_general: resolve_order(pick_entry_final.order_general),
        });
    }

    pick_entries.sort_by_key(|e| e.order_defence);
    for (idx, entry) in pick_entries.iter_mut().enumerate() {
        entry.order_defence = idx as i32 + 1;
    }

    pick_entries.sort_by_key(|e| e.order_offence);
    for (idx, entry) in pick_entries.iter_mut().enumerate() {
        entry.order_offence = idx as i32 + 1;
    }

    pick_entries.sort_by_key(|e| e.order_general);
    for (idx, entry) in pick_entries.iter_mut().enumerate() {
        entry.order_general = idx as i32 + 1;
    }

    Ok(PickLists { results: pick_entries })
}
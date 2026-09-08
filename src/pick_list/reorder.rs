use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use crate::entity::{pick_list, types::Team};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ReorderPick {
    pub team: Team,
    pub event_code: String,
    pub category: Category,
    pub direction: Direction,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum Category {
    Defence,
    Offence,
    General,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum Direction {
    Up,
    Down,
}

pub async fn reorder(data: ReorderPick, db: &DatabaseConnection) -> Result<(), DbErr> {
    let all_teams = pick_list::Entity::find()
        .filter(pick_list::Column::EventCode.eq(&data.event_code))
        .all(db)
        .await?;

    let mut pairs: Vec<(i32, i32)> = all_teams.iter()
        .filter_map(|t| {
            let order = match &data.category {
                Category::Defence => t.order_defence,
                Category::Offence => t.order_offence,
                Category::General => t.order_general,
            };
            order.map(|o| (t.id, o))
        })
        .collect();
    pairs.sort_by_key(|(_, o)| *o);

    let target_pos = pairs.iter().position(|(id, _)| {
        all_teams.iter().any(|t| t.id == *id
            && t.team == data.team.number
            && t.team_is_ab_team == data.team.is_ab_team)
    }).ok_or(DbErr::Custom("Team not found in pick list".to_string()))?;

    let new_pos = match data.direction {
        Direction::Up if target_pos > 0 => target_pos - 1,
        Direction::Down if target_pos < pairs.len() - 1 => target_pos + 1,
        _ => return Ok(()),
    };

    pairs.swap(target_pos, new_pos);

    for (idx, (team_id, _)) in pairs.iter().enumerate() {
        let model = all_teams.iter().find(|t| t.id == *team_id).unwrap();
        let mut active: pick_list::ActiveModel = model.clone().into();
        match &data.category {
            Category::Defence => active.order_defence = Set(Some(idx as i32 + 1)),
            Category::Offence => active.order_offence = Set(Some(idx as i32 + 1)),
            Category::General => active.order_general = Set(Some(idx as i32 + 1)),
        }
        active.update(db).await?;
    }

    Ok(())
}

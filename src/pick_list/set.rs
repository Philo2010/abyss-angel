use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use crate::entity::{pick_list, types::Team};



#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SetPick {
    pub team: Team,
    pub event_code: String,
    pub is_selected_defence: Option<bool>,
    pub is_selected_offence: Option<bool>,
    pub is_selected_general: Option<bool>,
}


pub async fn set(set_pick: SetPick, db: &DatabaseConnection) -> Result<(), DbErr> {
    let mut model_data: pick_list::ActiveModel = match pick_list::Entity::find()
        .filter(pick_list::Column::Team.eq(set_pick.team.number))
        .filter(pick_list::Column::TeamIsAbTeam.eq(set_pick.team.is_ab_team))
        .filter(pick_list::Column::EventCode.eq(set_pick.event_code))
        .one(db).await? {
            Some(a) => a.into(),
            None => {
                return Err(DbErr::Custom("Could not find the team!".to_string()));
            },
        };

    if let Some(is_selected_defence) = set_pick.is_selected_defence {
        model_data.is_selected_defence = sea_orm::Set(is_selected_defence);
    }
    if let Some(is_selected_offence) = set_pick.is_selected_offence {
        model_data.is_selected_offence = sea_orm::Set(is_selected_offence);
    }
    if let Some(is_selected_general) = set_pick.is_selected_general {
        model_data.is_selected_general = sea_orm::Set(is_selected_general);
    }

    model_data.update(db).await?;

    Ok(())
}
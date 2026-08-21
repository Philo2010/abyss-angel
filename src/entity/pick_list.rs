use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pick_list")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "item")]
    pub team: i32,
    #[sea_orm(unique_key = "item")]
    pub team_is_ab_team: bool,
    #[sea_orm(unique_key = "item")]
    pub event_code: String,
    pub is_selected_defence: bool,
    pub is_selected_offence: bool,
    pub is_selected_general: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {

}
impl ActiveModelBehavior for ActiveModel {}

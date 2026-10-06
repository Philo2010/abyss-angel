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
    // TODO: make these columns NOT NULL. Once every existing row is guaranteed to
    // have an order (backfilled by migrate_7410465), these can become `i32`. Until
    // then reorder.rs skips teams with a NULL order for the category, which can
    // leave gaps/conflicts in the persisted ordering. Making them non-null also
    // lets get.rs and reorder.rs drop their Option handling.
    pub order_defence: Option<i32>,
    pub order_offence: Option<i32>,
    pub order_general: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {

}
impl ActiveModelBehavior for ActiveModel {}

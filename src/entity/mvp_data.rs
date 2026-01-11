use sea_orm::entity::prelude::*;



#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "mvp_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub mvp_team: i32,
    pub mvp_is_ab_team: bool,
    pub comment: String,
    pub is_blue: bool,
    pub total_score: i32,
    pub penalty_score: i32,  //Given by op teams
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {

}

impl ActiveModelBehavior for ActiveModel {}
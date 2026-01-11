use uuid::Uuid;
use sea_orm::entity::prelude::*;


use crate::entity::mvp_data;



#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "mvp_scouts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub scouter: Uuid,

    pub is_blue: bool,

    //Data for MVP -> Will be found when MVP scouts
    #[sea_orm(has_one)]
    pub data: Option<i32>
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
 
}
impl ActiveModelBehavior for ActiveModel {}
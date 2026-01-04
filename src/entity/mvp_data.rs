use sea_orm::entity::prelude::*;



#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "mvp_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub mvp_team: i32,
    pub mvp_is_ab_team: bool,
    pub comment: String,
    pub total_score_for_red: i32,
    pub total_score_for_blue: i32,
    pub penalty_score_for_red: i32,  //Given by blue
    pub penalty_score_for_blue: i32, //Given by red
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::mvp_scouters::Entity",
        from = "Column::Id",
        to = "super::mvp_scouters::Column::Data"
    )]
    MvpScouters,
}


impl Related<super::mvp_scouters::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MvpScouters.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
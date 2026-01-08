pub const YEAR: i32 = 9999;

use schemars::JsonSchema;
use sea_orm::{ActiveValue::{NotSet, Set}, entity::prelude::*};
use serde::{Deserialize, Serialize};
use crate::pit::pit::{PitEditSpecific, PitInsertsSpecific, PitScoutStandard, PitSpecific};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, JsonSchema)]
#[sea_orm(table_name = "genertic_header")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub width: i32,
    pub height: i32
}


#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Insert {
    pub width: i32,
    pub height: i32
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Edit {
    pub width: Option<i32>,
    pub height: Option<i32>
}

pub struct Functions;


#[async_trait]
impl PitScoutStandard for Functions {
    async fn insert_game_specific(&self, data: PitInsertsSpecific, db: &DatabaseConnection) -> Result<i32, DbErr> {
        match data {
            PitInsertsSpecific::ExamplePit(a) => {
               let active = ActiveModel { 
                    id: NotSet,
                    width: Set(a.width),
                    height: Set(a.height), 
                };
                let res = Entity::insert(active).exec(db).await?.last_insert_id;
                return Ok(res);
            },
            _ => {
                return Err(DbErr::Custom("Incorrect year!".to_string()));
            }
        }
    }
    async fn get_pit_specific(&self, id: i32, db: &DatabaseConnection) -> Result<PitSpecific, DbErr> {
        let game_data = Entity::find_by_id(id).one(db).await?.ok_or(DbErr::RecordNotFound("Failed to find pit data".to_string()))?;
        let res = PitSpecific::ExamplePit(game_data);
        Ok(res)
    }
    async fn edit_pit(&self, id: i32, data: PitEditSpecific, db: &DatabaseConnection) -> Result<(), DbErr> {
        let data_u = match data {
            PitEditSpecific::ExamplePit(a) => {
                a
            },
            _ => {
                return Err(DbErr::RecordNotFound("Not the correct year!".to_string()));
            } 
        };
        
        let active = ActiveModel {
            id: Set(id),
            width: data_u.width.map(Set).unwrap_or(NotSet),
            height: data_u.height.map(Set).unwrap_or(NotSet) 
        };
        let res = Entity::update(active).exec(db).await?;

        Ok(())
    }
    fn get_type_year(&self) -> i32 {
        return YEAR;
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
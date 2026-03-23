pub const YEAR: i32 = 2026;

use schemars::JsonSchema;
use sea_orm::{ActiveValue::{NotSet, Set}, entity::prelude::*};
use serde::{Deserialize, Serialize};
use crate::{backenddb::entrys::rebuilt::ClimbState, pit::pit::{PitEditSpecific, PitInsertsSpecific, PitScoutStandard, PitSpecific}};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PathPoint {
    pub x: i32,
    pub y: i32,
}

pub type Stroke = Vec<PathPoint>;
pub type AutoPath = Vec<Stroke>;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, JsonSchema)]
#[sea_orm(table_name = "pit_rebuilt")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub is_swerve: bool,
    //is the gear if is_swerve is true, is drivebase if false
    pub gear_or_drivebase: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub auto_paths: Vec<Json>,
    pub years_of_driver_experience: i32,
    pub what_they_are_looking_from_the_tournament: String,
    pub do_they_have_a_scouter: bool,
    pub how_many_batteries_do_they_have_on_hand: i32,
    pub willing_to_play_defence: bool,
    pub strategy: String,
    pub can_pass: bool,
    pub can_shoot: bool,
    pub climb: ClimbState

}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
enum Drivebase {
    Swerve(String), //gear
    Other(String) //name of other drivebase
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Insert {
    pub drivebase: Drivebase,
    pub auto_paths: Vec<AutoPath>,
    pub years_of_driver_experience: i32,
    pub what_they_are_looking_from_the_tournament: String,
    pub do_they_have_a_scouter: bool,
    pub how_many_batteries_do_they_have_on_hand: i32,
    pub willing_to_play_defence: bool,
    pub strategy: String,
    pub can_pass: bool,
    pub can_shoot: bool,
    pub climb: ClimbState
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Edit {
    pub drivebase: Option<Drivebase>,
    pub auto_paths: Option<Vec<AutoPath>>,
    pub years_of_driver_experience: Option<i32>,
    pub what_they_are_looking_from_the_tournament: Option<String>,
    pub do_they_have_a_scouter: Option<bool>,
    pub how_many_batteries_do_they_have_on_hand: Option<i32>,
    pub willing_to_play_defence: Option<bool>,
    pub strategy: Option<String>,
    pub can_pass: Option<bool>,
    pub can_shoot: Option<bool>,
    pub climb: Option<ClimbState>
}

pub struct Functions;


#[async_trait]
impl PitScoutStandard for Functions {
    async fn insert_game_specific(&self, data: PitInsertsSpecific, db: &DatabaseConnection) -> Result<i32, DbErr> {
        #[allow(unreachable_patterns)]
        match data {
            PitInsertsSpecific::RebuiltPit(data) => {
                let is_swerve;
                let gear_or_drivebase;

                if data.auto_paths.len() > 3 {
                    return Err(DbErr::Custom("Invaild amount of Auth Paths".to_string()));
                }

                match data.drivebase {
                    Drivebase::Swerve(a) => {
                        is_swerve = true;
                        gear_or_drivebase = a;
                    },
                    Drivebase::Other(a) => {
                        is_swerve = false;
                        gear_or_drivebase = a;
                    },
                }

                let active = ActiveModel {
                    id: NotSet,
                    is_swerve: Set(is_swerve),
                    gear_or_drivebase: Set(gear_or_drivebase),
                    auto_paths: Set(data.auto_paths.into_iter().map(|p| serde_json::to_value(p).unwrap()).collect()),
                    years_of_driver_experience: Set(data.years_of_driver_experience),
                    what_they_are_looking_from_the_tournament: Set(data.what_they_are_looking_from_the_tournament),
                    do_they_have_a_scouter: Set(data.do_they_have_a_scouter),
                    how_many_batteries_do_they_have_on_hand: Set(data.how_many_batteries_do_they_have_on_hand),
                    willing_to_play_defence: Set(data.willing_to_play_defence),
                    strategy: Set(data.strategy),
                    can_pass: Set(data.can_pass),
                    can_shoot: Set(data.can_shoot),
                    climb: Set(data.climb)
                };
                let res = crate::pit::entrys::pit_rebuilt::Entity::insert(active).exec(db).await?.last_insert_id;
                return Ok(res);
            }
            _ => {
                return Err(DbErr::Custom("Incorrect year!".to_string()));
            }
        }
    }
    async fn get_pit_specific(&self, id: i32, db: &DatabaseConnection) -> Result<PitSpecific, DbErr> {
        let game_data = Entity::find_by_id(id).one(db).await?.ok_or(DbErr::RecordNotFound("Failed to find pit data".to_string()))?;
        let res = PitSpecific::RebuiltPit(game_data);
        Ok(res)
    }
    async fn edit_pit(&self, id: i32, data: PitEditSpecific, db: &DatabaseConnection) -> Result<(), DbErr> {
        #[allow(unreachable_patterns)]
        let data_u = match data {
            PitEditSpecific::RebuiltPit(a) => {
                a
            },
            _ => {
                return Err(DbErr::RecordNotFound("Not the correct year!".to_string()));
            } 
        };

        let is_swerve;
        let gear_or_drivebase;

        if let Some(auto_paths) = &data_u.auto_paths {
            if auto_paths.len() > 3 {
                return Err(DbErr::Custom("Invaild amount of Auth Paths".to_string()));
            }
        }

        match data_u.drivebase {
            Some(a) => {
                match a {
                    Drivebase::Swerve(a) => {
                        is_swerve = Set(true);
                        gear_or_drivebase = Set(a);
                    },
                    Drivebase::Other(a) => {
                        is_swerve = Set(false);
                        gear_or_drivebase = Set(a);
                    },
                }
            },
            None => {
                is_swerve = NotSet;
                gear_or_drivebase = NotSet;
            },
        }
        

        let active = ActiveModel {
            id:Set(id),
            is_swerve: is_swerve,
            gear_or_drivebase: gear_or_drivebase,
            auto_paths: data_u.auto_paths.map(|paths| Set(paths.into_iter().map(|p| serde_json::to_value(p).unwrap()).collect())).unwrap_or(NotSet),
            years_of_driver_experience: data_u.years_of_driver_experience.map(Set).unwrap_or(NotSet),
            what_they_are_looking_from_the_tournament: data_u.what_they_are_looking_from_the_tournament.map(Set).unwrap_or(NotSet),
            do_they_have_a_scouter: data_u.do_they_have_a_scouter.map(Set).unwrap_or(NotSet),
            how_many_batteries_do_they_have_on_hand: data_u.how_many_batteries_do_they_have_on_hand.map(Set).unwrap_or(NotSet),
            willing_to_play_defence: data_u.willing_to_play_defence.map(Set).unwrap_or(NotSet),
            strategy: data_u.strategy.map(Set).unwrap_or(NotSet),
            can_pass: data_u.can_pass.map(Set).unwrap_or(NotSet),
            can_shoot: data_u.can_shoot.map(Set).unwrap_or(NotSet),
            climb: data_u.climb.map(Set).unwrap_or(NotSet)
        };
        let _res = Entity::update(active).exec(db).await?;

        Ok(())
    }
    fn get_type_year(&self) -> i32 {
        YEAR
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
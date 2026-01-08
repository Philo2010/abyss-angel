
/* 
pub struct Model {
    #[sea_orm(primary_key)]
    pub created_at: DateTime,
} */

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use rocket::Data;
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{SETTINGS, define_pits, entity::pit_header, pit::pit_example};


#[async_trait]
pub trait PitScoutStandard: Send {
    async fn insert_game_specific(&self, data: PitInsertsSpecific, db: &DatabaseConnection) -> Result<i32, DbErr>;
    async fn get_pit_specific(&self, id: i32, db: &DatabaseConnection) -> Result<PitSpecific, DbErr>;
    async fn edit_pit(&self, id: i32, data: PitEditSpecific, db: &DatabaseConnection) -> Result<(), DbErr>;
    fn get_type_year(&self) -> i32;
}

pub struct PitHeaderInsert {
    pub user: Uuid,
    pub team: i32,
    pub is_ab_team: bool,
    pub event_code: String,
}

pub struct PitInsert {
    pub header: PitHeaderInsert,
    pub pit: PitInsertsSpecific
}

#[derive(Serialize)]
pub struct FullHeader {
    pub id: i32,
    pub user: Uuid,
    pub team: i32,
    pub is_ab_team: bool,
    pub event_code: String,
    pub created_at: DateTime<Local>,
}

#[derive(Serialize)]
pub struct PitGet {
    pub header: FullHeader,
    pub pit: PitSpecific
}

async fn prim_pit_insert(data: PitInsert, db: &DatabaseConnection, model: Box<dyn PitScoutStandard>) -> Result<i32, DbErr> {
    let spec_id = model.insert_game_specific(data.pit, db).await?;

    let created_at: NaiveDateTime = Local::now().naive_local();

    let active_head: pit_header::ActiveModel = pit_header::ActiveModel { 
        id: NotSet,
        user: Set(data.header.user),
        team: Set(data.header.team),
        is_ab_team: Set(data.header.is_ab_team),
        event_code: Set(data.header.event_code),
        created_at: Set(created_at),
        pit_data: Set(spec_id),
        pit_type: Set(model.get_type_year())
    };

    let res = active_head.insert(db).await?.id;

    Ok(res)
}

async fn prim_pit_edit(data: PitEditSpecific, db: &DatabaseConnection, id: i32, model: Box<dyn PitScoutStandard>) -> Result<(), DbErr> {
    model.edit_pit(id, data, db).await
}

async fn prim_pit_get(team: i32, is_ab_team: bool, event_code: &String, db: &DatabaseConnection, model: Box<dyn PitScoutStandard>) -> Result<PitGet, DbErr> {
    let header_data = pit_header::Entity::find()
        .filter(pit_header::Column::Team.eq(team))
        .filter(pit_header::Column::IsAbTeam.eq(is_ab_team))
        .filter(pit_header::Column::EventCode.eq(event_code))
        .one(db).await?.ok_or(DbErr::RecordNotFound("Could not find pit header!".to_string()))?;

    let pit_data = model.get_pit_specific(header_data.pit_data, db).await?;
    let created_at = Local.from_local_datetime(&header_data.created_at).single().ok_or(DbErr::Custom("Failed to parse date".to_string()))?;
    let header: FullHeader = FullHeader { 
        id: header_data.id,
        user: header_data.user,
        team: header_data.team,
        is_ab_team: header_data.is_ab_team,
        event_code: header_data.event_code,
        created_at: created_at
    };
    let total_data = PitGet {
        header: header,
        pit: pit_data
    };

    Ok(total_data)
}

pub async fn pit_get(team: i32, is_ab_team: bool, event_code: &String, db: &DatabaseConnection) -> Result<PitGet, DbErr> {
    let model = pit_dispatch(SETTINGS.year);
    prim_pit_get(team, is_ab_team, event_code, db, model).await
}

pub async fn pit_insert(data: PitInsert, db: &DatabaseConnection) -> Result<i32, DbErr> {
    let model = pit_dispatch(SETTINGS.year);
    prim_pit_insert(data, db, model).await
}

pub async fn pit_edit(data: PitEditSpecific, db: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    let model = pit_dispatch(SETTINGS.year);
    prim_pit_edit(data, db, id, model).await
}


define_pits!(
    ExamplePit => crate::pit::pit_example,
);
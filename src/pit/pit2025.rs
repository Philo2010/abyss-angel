use std::error::Error;
use std::pin::Pin;

use rocket::http::uri::Query;
use rocket::tokio;
use sea_orm::ActiveValue::{NotSet, Unchanged};
use sea_orm::sqlx::types::chrono::Utc;
use sea_orm::{ActiveValue::Set, FromQueryResult, QuerySelect, Schema, entity, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sea_orm::sea_query::{Alias, Expr, Func, Mode, SelectStatement};
use sea_orm::{Database, DatabaseBackend, StatementBuilder, query::*};
use phf::phf_map;


pub const PITYEARSINSERT: phf::Map<i32, for<'a> fn(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, i32>> = phf_map! {
    2025i32 =>  Model::insert
};

pub const PITYEARDELETE: phf::Map<i32, fn(db: & DatabaseConnection, id: i32) -> BoxFuture<()>> = phf_map! {
    2025i32 => Model::delete_scout,
};

pub const PITYEARSGET: phf::Map<i32, fn(db: & DatabaseConnection, id: i32) -> BoxFuture<serde_json::Value>> = phf_map! {
    2025i32 => Model::get_scout,
};

pub const PITYEARSEDIT: phf::Map<i32, for<'a> fn(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, ()>> = phf_map! {
    2025i32 => Model::edit,
};

pub const PITYEARSGETEVENTTEAM: phf::Map<i32, for<'a> fn(db: &'a DatabaseConnection, team: i32, event: &'a str) -> BoxFuture<'a, serde_json::Value>> = phf_map!(
    2025i32 => Model::get_by_event_team,
);

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, Box<dyn Error>>> + Send + 'a>>;

use crate::pit::pit2025;
use crate::{SETTINGS, boxed_async, user};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "pit2025")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    
    pub user: String,
    pub team: i32,
    pub event_code: String,
    #[sea_orm(auto_create_time)] 
    pub created_at: DateTime,

    //Per year
    pub height: i32,
    pub width: i32,
}

#[derive(Debug, Clone, EnumIter, DeriveRelation)]
pub enum Relation {
}



impl ActiveModelBehavior for ActiveModel {
    
}

trait PitScoutYear {
    fn insert<'a>(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, i32>;
    fn get_scout<'a>(db: &'a DatabaseConnection, id: i32) -> BoxFuture<'a, serde_json::Value>;
    fn delete_scout<'a>(db: &'a DatabaseConnection, id: i32) -> BoxFuture<'a, ()>;
    fn edit<'a>(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, ()>;
    fn get_by_event_team<'a>(db: &'a DatabaseConnection, team: i32, event: &'a str) -> BoxFuture<'a, serde_json::Value>;
}

impl Model {
    /// Async function to create the table
    pub async fn create_table_postgres(database_url: &str) -> Result<(), sea_orm::DbErr> {
        let db = sea_orm::Database::connect(database_url).await?;
        let schema = Schema::new(sea_orm::DatabaseBackend::Postgres);
        let mut stmt = schema.create_table_from_entity(Entity);
        stmt.if_not_exists();
        db.execute(&stmt).await?;
        Ok(())
    }
}

impl ActiveModel {
    fn debug_set_from_json(&mut self, json: &serde_json::Value) {
        let get_string = |key: &str| -> String {
            json.get(key).map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default()
        };
    
        self.user = Set(get_string("user"));
        self.team = Set(json.get("team").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.event_code = Set(get_string("event_code"));
        self.height = Set(json.get("height").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.width = Set(json.get("width").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
    }
    fn debug_set_from_json_full(&mut self, json: &serde_json::Value) {
        let a = match json.get("id").and_then(|v| v.as_i64()) {
            Some(a) => Set(a as i32),
            None => {
                NotSet
            },
        };

        self.id = a;
        self.user = Set(json.get("user").and_then(|v| v.as_str()).unwrap_or("").to_string());
        self.team = Set(json.get("team").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.event_code = Set(json.get("event_code").and_then(|v| v.as_str()).unwrap_or("").to_string());
        self.height = Set(json.get("height").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.width = Set(json.get("width").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
    }

}

impl PitScoutYear for Model {

    fn get_by_event_team<'a>(db: &'a DatabaseConnection, team: i32, event: &'a str) -> BoxFuture<'a, serde_json::Value> {
        boxed_async!(async move {
            let model = pit2025::Entity::find()
            .filter(pit2025::Column::Team.eq(team))
            .filter(pit2025::Column::EventCode.contains(event))
            .one(db)
            .await
            .map_err(|e| -> Box<dyn Error> { e.into() })?
            .ok_or_else(|| -> Box<dyn Error> { "Could not find pit data".into() })?;
        
            // Convert model to JSON and return Ok
            let json_value = serde_json::to_value(&model)?;
            Ok(json_value)
        })
    }

    fn edit<'a>(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, ()> {
        boxed_async!(async move {

            let mut active_model = pit2025::ActiveModel { ..Default::default() };
            let a = active_model.debug_set_from_json_full(&json);
            
            println!("{:?}", active_model);

            active_model.update(db).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            Ok(())
        })
    }

    fn delete_scout<'a>(db: &'a DatabaseConnection, id: i32) -> BoxFuture<'a, ()> {
        boxed_async!(async move {
            user::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            Ok(())
        })
    }

    fn insert<'a>(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, i32> {
        boxed_async!(async move {
            println!("{json}");
            let mut active_model = pit2025::ActiveModel { ..Default::default() };
            let a = active_model.debug_set_from_json(&json);
            println!("{:?}", active_model);
            active_model.created_at = Set(Utc::now().naive_utc());
            

            let inserted = ActiveModel::insert(active_model, db).await?;
            Ok(inserted.id)
        })
    }


    fn get_scout<'a>(db: &'a DatabaseConnection, id: i32) -> BoxFuture<'a, serde_json::Value>{
        Box::pin(async move {
            let model = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Box::<dyn std::error::Error>::from("Record not found"))?;
            
            Ok(serde_json::to_value(model)?)
        })
    }
}

#[ctor::ctor]
fn init_tables() {
    let name = Entity.table_name();
    println!("SCHEMA CREATOR: CREATE TABLE {name}");
    let database_url = SETTINGS.db_path.to_string();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        Model::create_table_postgres(&database_url)
            .await
            .unwrap_or_else(|e| panic!("Failed to create table: {}", e));
    });
}
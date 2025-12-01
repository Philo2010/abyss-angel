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


pub const YEARSINSERT: phf::Map<i32, for<'a> fn(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, i32>> = phf_map! {
    2025i32 =>  Model::insert
};

pub const YEARSGRAPH: phf::Map<i32, fn(event: Option<String>, team: i32, db: &DatabaseConnection) -> BoxFuture<serde_json::Value>> = phf_map! {
    2025i32 =>  Model::graph
};

pub const YEARSAVG: phf::Map<i32, fn(event: Option<String>, db: & DatabaseConnection) -> BoxFuture<serde_json::Value>> = phf_map! {
    2025i32 =>  Model::averages
};

pub const YEARSSEARCH: phf::Map<i32, fn(event: Option<String>, scouter: Option<String>, team: Option<i32>, db: & DatabaseConnection) -> BoxFuture<serde_json::Value>> = phf_map! {
    2025i32 =>  Model::search,
};

pub const YEARDELETE: phf::Map<i32, fn(db: & DatabaseConnection, id: i32) -> BoxFuture<()>> = phf_map! {
    2025i32 => Model::delete_scout,
};

pub const YEARSGET: phf::Map<i32, fn(db: & DatabaseConnection, id: i32) -> BoxFuture<serde_json::Value>> = phf_map! {
    2025i32 => Model::get_scout,
};

pub const YEARSEDIT: phf::Map<i32, for<'a> fn(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, ()>> = phf_map! {
    2025i32 => Model::edit,
};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, Box<dyn Error>>> + Send + 'a>>;

use crate::{SETTINGS, boxed_async, user};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    // Common header
    
    //#[serde(rename = "User")]
    pub user: String, // Foreign key to user.id
    //#[serde(rename = "Team")]
    pub team: i32,
    //#[serde(rename = "Match ID")]
    pub matchid: i32,
    //#[serde(rename = "Set")]
    pub set: i32,

    #[serde(skip_deserializing)]
    //#[serde(rename = "Total Score")]
    pub total_score: i32,
    //#[serde(rename = "Event Code")]
    pub event_code: String,
    //#[serde(rename = "Tournament Level")]
    pub tournament_level: String,
    //#[serde(rename = "Station")]
    pub station: String,
    //#[serde(rename = "Created At")]
    #[sea_orm(auto_create_time)] 
    pub created_at: DateTime,

    //#[serde(rename = "Hehe")]
    pub hehe: i32,
    //#[serde(rename = "Hoohoo")]
    pub hoohoo: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}


impl ActiveModelBehavior for ActiveModel {
    
}

trait ScoutYear {
    fn insert<'a>(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, i32>;
    fn search<'a>(event: Option<String>, scouter: Option<String>, team: Option<i32>, db: &'a DatabaseConnection) -> BoxFuture<'a, serde_json::Value>;
    fn averages<'a>(event: Option<String>, db: &'a DatabaseConnection) -> BoxFuture<'a, serde_json::Value>;
    fn graph<'a>(event: Option<String>, team: i32, db: &'a DatabaseConnection) -> BoxFuture<'a, serde_json::Value>;
    fn get_scout<'a>(db: &'a DatabaseConnection, id: i32) -> BoxFuture<'a, serde_json::Value>;
    fn delete_scout<'a>(db: &'a DatabaseConnection, id: i32) -> BoxFuture<'a, ()>;
    fn edit<'a>(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, ()>;
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
    ///TODO Must create auto total  
    pub fn total(&mut self) {
        let hehe = match self.hehe {
            Set(a) | Unchanged(a) => a,
            entity::ActiveValue::NotSet => 0,
        };

        let hoohoo = match self.hoohoo {
            Set(a) | Unchanged(a) => a,
            entity::ActiveValue::NotSet => 0,
        };

        self.total_score = Set(hehe + hoohoo);
    }
    fn debug_set_from_json(&mut self, json: &serde_json::Value) {
        self.user = Set(json.get("user").and_then(|v| v.as_str()).unwrap_or("").to_string());
        self.team = Set(json.get("team").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.matchid = Set(json.get("matchid").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.set = Set(json.get("set").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.event_code = Set(json.get("event_code").and_then(|v| v.as_str()).unwrap_or("").to_string());
        self.tournament_level = Set(json.get("tournament_level").and_then(|v| v.as_str()).unwrap_or("").to_string());
        self.station = Set(json.get("station").and_then(|v| v.as_str()).unwrap_or("").to_string());
        self.hehe = Set(json.get("hehe").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.hoohoo = Set(json.get("hoohoo").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
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
        self.matchid = Set(json.get("matchid").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.set = Set(json.get("set").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.event_code = Set(json.get("event_code").and_then(|v| v.as_str()).unwrap_or("").to_string());
        self.tournament_level = Set(json.get("tournament_level").and_then(|v| v.as_str()).unwrap_or("").to_string());
        self.station = Set(json.get("station").and_then(|v| v.as_str()).unwrap_or("").to_string());
        self.hehe = Set(json.get("hehe").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
        self.hoohoo = Set(json.get("hoohoo").and_then(|v| v.as_i64()).unwrap_or(0) as i32);
    }

}

impl ScoutYear for Model {

    fn edit<'a>(db: &'a DatabaseConnection, json: &'a serde_json::Value) -> BoxFuture<'a, ()> {
        boxed_async!(async move {

            let mut active_model = user::ActiveModel { ..Default::default() };
            let a = active_model.debug_set_from_json_full(&json);

            active_model.total();

            
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
            let mut active_model = user::ActiveModel { ..Default::default() };
            let a = active_model.debug_set_from_json(&json);


            active_model.total();
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
    
    fn search<'a>(event: Option<String>, scouter: Option<String>, team: Option<i32>, db: &'a DatabaseConnection) -> BoxFuture<'a, serde_json::Value> {
        use user::Entity; // bring the generated Entity into scope
        use user::Column;

        // Start building the query
        let mut query = Entity::find();

        // Apply optional filters
        if let Some(event_code) = event {
            query = query.filter(Column::EventCode.contains(&event_code));
        }

        if let Some(scouter) = scouter {
            query = query.filter(Column::User.eq(scouter));
        }

        if let Some(team_id) = team {
            query = query.filter(Column::Team.eq(team_id));
        }

        boxed_async!(async move {
            // Execute the query
            let results = query.all(db).await?;
            let res = serde_json::to_value(&results)?;

            Ok(res)
        })
    }

    fn averages<'a>(event: Option<String>, db: &'a DatabaseConnection) -> BoxFuture<'a, serde_json::Value> {

        //Create the query
        let query: SelectStatement = if let Some(eve) = event {
            sea_orm::sea_query::Query::select()
            .expr_as(Expr::col(user::Column::Team),"team")
            .expr_as(Func::avg(Expr::col(user::Column::Hehe)).cast_as(Alias::new("DOUBLE PRECISION")), "avg_hehe")
            .expr_as(Func::avg(Expr::col(user::Column::Hoohoo)).cast_as(Alias::new("DOUBLE PRECISION")), "avg_hoohoo")
            .expr_as(Func::avg(Expr::col(user::Column::TotalScore)).cast_as(Alias::new("DOUBLE PRECISION")), "avg_total")
            .from(Entity)
            .and_where(Column::EventCode.eq(eve))
            .group_by_col(user::Column::Team)
            .to_owned()
        } else {
            sea_orm::sea_query::Query::select()
            .expr_as(Expr::col(user::Column::Team),"team")
            .expr_as(Func::avg(Expr::col(user::Column::Hehe).cast_as(Alias::new("DOUBLE PRECISION"))), "avg_hehe")
            .expr_as(Func::avg(Expr::col(user::Column::Hoohoo)).cast_as(Alias::new("DOUBLE PRECISION")), "avg_hoohoo")
            .expr_as(Func::avg(Expr::col(user::Column::TotalScore)).cast_as(Alias::new("DOUBLE PRECISION")), "avg_total")
            .from(Entity)
            .group_by_col(user::Column::Team)
            .to_owned()   
        };

        boxed_async!(async move {
            // Run the query
            let row = db.query_all(&query).await?;

            //Convert to json
            println!("{:?}", row);
            let result = row.into_iter().map(|r| {
                json!({
                    "User": r.try_get_by::<i32, _>(0).unwrap_or(0),
                    "Hehe": r.try_get_by::<f64, _>(1).unwrap_or(0.0),
                    "Hoohoo": r.try_get_by::<f64, _>(2).unwrap_or(0.0),
                    "Total_score": r.try_get_by::<f64, _>(3).unwrap_or(0.0),
                })
            }).collect();

            Ok(result)
        })
    }

    fn graph<'a>(event: Option<String>, team: i32, db: &'a DatabaseConnection) -> BoxFuture<'a, serde_json::Value> {
        boxed_async!(async move {
            let models = if let Some(eve) = event {
                Entity::find()
                    .filter(Column::EventCode.contains(eve))
                    .filter(Column::Team.eq(team)).all(db).await?
            } else {
                Entity::find()
                .filter(Column::Team.eq(team)).all(db).await?
            };
            //Custom code needs to be genertated for this

            let json_vec: Vec<_> = models.into_iter().map(|m| {
                json!({
                    "Total Score": m.total_score,
                    "HehePoints": m.hehe,
                    "HoohooPoints": m.hoohoo,
                    "Time": m.created_at
                })
            }).collect();

            let json_output = serde_json::Value::Array(json_vec);

            Ok(json_output)
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
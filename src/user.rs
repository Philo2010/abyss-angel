use std::fmt::Error;

use rocket::tokio;
use sea_orm::{ActiveValue::Set, Schema, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::json;


use crate::user;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct  Model {
    #[sea_orm(primary_key)]
    id: i32,
    name: String,
    hehe: i32,
}


#[derive(Debug, Clone, EnumIter, DeriveRelation)]
pub enum Relation {
    //No relation needed, emtiy
}


impl ActiveModelBehavior for ActiveModel {
    
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
    
    pub async fn insert(db: &DatabaseConnection, json: serde_json::Value) -> Result<i32, Box<dyn std::error::Error>> {
        
        let mut active_model = user::ActiveModel {
            ..Default::default()
        };

        let _a = active_model.set_from_json(json)?;

        let inserted = ActiveModel::insert(active_model, db).await?;

        Ok(inserted.id)
    }

    pub async fn search (event: String, db: &DatabaseConnection) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        todo!()
    }

}

#[ctor::ctor]
fn init_tables() {
    let name = Entity.table_name();
    println!("SCHEMA CREATOR: CREATE TABLE {name}");
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/testdb".to_string());
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        Model::create_table_postgres(&database_url)
            .await
            .unwrap_or_else(|e| panic!("Failed to create table: {}", e));
    });
}
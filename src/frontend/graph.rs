use std::error::Error;

use rocket::State;
use rocket::form::Form;
use rocket::{post, serde::json::Json};
use sea_orm::DatabaseConnection;
use serde_json::{Value, json};

use crate::SETTINGS;
use crate::user::{YEARSGRAPH, YEARSINSERT};

#[derive(FromForm)]
struct GraphForm {
    event: Option<String>,
    teams: Vec<i32>
}


#[post("/graph", data = "<body>")]
async fn graph(body: Form<GraphForm>, db: &State<DatabaseConnection>) -> String {
    //Get the function
    let insrfunc = YEARSGRAPH[&SETTINGS.year];
    
    //Check values
    if body.teams.is_empty() {
        return "Team is empty".to_string();
    }

    let mut team_data: Vec<Value> = Vec::new();

    for team in body.teams.iter() {
        let mut e = match insrfunc(body.event.clone(), team.clone(), db).await {
            Ok(a) => a,
            Err(_) => {continue;},
        };

        if let Value::Array(ref mut vec) = e {
            vec.insert(0, json!(team));
        }
        team_data.push(e);
    }

    let json_array = Value::Array(team_data);

    json_array.to_string()
}

use rocket::{State, data::ToByteUnit, form::Form, http::CookieJar, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};
use serde_json::Value;

use crate::{auth, upcoming_handler::upcoming_team};

#[derive(Debug, FromForm)]
pub struct AssignmentForm {
    #[field(name = "assignments")]
    pub assignments: Vec<ScoutAssignment>,
}

#[derive(Debug, FromForm)]
pub struct ScoutAssignment {
    pub team_match_id: i32,
    pub scouter: String,
}




#[post("/assign_scouts", data = "<data>")]
pub async fn assign_scout(db: &State<DatabaseConnection>, data: Form<AssignmentForm>, cookies: &CookieJar<'_>) -> Template {

    if !(auth::check::check(cookies, db).await) {
        return Template::render("error", context! {error: "Failed to auth (are you logined?)"});
    }

    for assignment in &data.assignments {
        if !assignment.scouter.trim().is_empty() {
            println!("Team Match {} -> Scouter {}", assignment.team_match_id, assignment.scouter);
            let match_team_data = match upcoming_team::Entity::find_by_id(assignment.team_match_id).one(db.inner()).await {
                Ok(Some(a)) => a,
                Ok(None) => {
                    return Template::render("error", context! {error: "Error! Could not find the data for that match! Please try refreshing your page and if that fails contact devs"});
                }
                Err(a) => {
                    return Template::render("error", context! {error: format!("Error! Failed to retrive Team data from sever: {a}")});
                },
            };

            let mut match_team: upcoming_team::ActiveModel = match_team_data.into();

            match_team.scouter = Set(Some(assignment.scouter.clone()));

            match upcoming_team::Entity::update(match_team).exec(db.inner()).await {
                Ok(_) => {},
                Err(a) => {
                    return Template::render("error", context! {error: format!("Error! Failed to insert data into database: {a}")});
                },
            };

        }
    }


    Template::render("suc", context! {message: "Done!"})
}
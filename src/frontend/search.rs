
use rocket::State;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::post;
use rocket::serde::json::Json;
use rocket_dyn_templates::{Template, context};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_json::Value;

use crate::backenddb::game::{SearchParam, search_game};
use crate::entity::sea_orm_active_enums::{Stations, TournamentLevels};
use crate::frontend::ApiResult;
use crate::{SETTINGS, auth, sexymac};


#[derive(Deserialize, JsonSchema)]
pub struct SearchParamData {
    //Id should be done via get
    pub user: Option<String>,
    pub team: Option<i32>,
    pub is_ab_team: Option<bool>,
    pub match_id: Option<i32>,
    pub set: Option<i32>,
    pub total_score: Option<i32>,
    pub event_code: Option<String>,
    pub tournament_level: Option<TournamentLevels>,
    pub station: Option<Stations>,
    pub is_mvp: Option<bool>
}

impl Into<SearchParam> for SearchParamData {
    fn into(self) -> SearchParam {
        SearchParam { 
            user: self.user,
            team: self.team,
            is_ab_team: self.is_ab_team,
            match_id: self.match_id,
            set: self.set,
            total_score: self.total_score,
            event_code: self.event_code,
            tournament_level: self.tournament_level,
            station: self.station,
            year: SETTINGS.year,
            is_mvp: self.is_mvp
        }
    }
}


#[rocket_okapi::openapi]
#[post("/api/search", data="<body>")]
pub async fn search(body: Json<SearchParamData>, db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Json<ApiResult<Vec<crate::backenddb::game::GamesFull>>> {
    if !auth::check::check(cookies, db).await {
        return Json(ApiResult::Error("Need to be admin!".to_string()));
    }
    let data: SearchParam = body.into_inner().into();

    let a: Vec<crate::backenddb::game::GamesFull> = match search_game(&data, db).await {
        Ok(a) => {
            return Json(ApiResult::Success(a));
        },
        Err(a) => {
            return Json(ApiResult::Error(a.to_string()));
        },
    };
}
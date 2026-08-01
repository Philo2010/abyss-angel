
use rocket::State;
use rocket::http::CookieJar;
use rocket::post;
use rocket::serde::json::Json;
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::backenddb::game::{SearchParam, search_game};
use crate::entity::sea_orm_active_enums::{Stations, TournamentLevels};
use crate::frontend::ApiResult;
use crate::snowgrave::datatypes::Team;
use crate::{SETTINGS, auth};


#[derive(Deserialize, JsonSchema)]
pub struct SearchParamData {
    //Id should be done via get
    pub user: Option<String>,
    pub teams: Option<Vec<Team>>,
    pub match_id: Option<i32>,
    pub set: Option<i32>,
    pub total_score: Option<i32>,
    pub event_code: Option<String>,
    pub tournament_level: Option<TournamentLevels>,
    pub station: Option<Stations>,
    pub is_mvp: Option<bool>,
    pub include_midway: Option<bool>,
}

impl From<SearchParamData> for SearchParam {
    fn from(val: SearchParamData) -> Self {
        SearchParam {
            user: val.user,
            teams: val.teams,
            match_id: val.match_id,
            set: val.set,
            total_score: val.total_score,
            event_code: val.event_code,
            tournament_level: val.tournament_level,
            station: val.station,
            year: SETTINGS.year,
            is_mvp: val.is_mvp,
            include_midway: val.include_midway,
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

    let _a: Vec<crate::backenddb::game::GamesFull> = match search_game(&data, db).await {
        Ok(a) => {
            return Json(ApiResult::Success(a));
        },
        Err(a) => {
            return Json(ApiResult::Error(a.to_string()));
        },
    };
}
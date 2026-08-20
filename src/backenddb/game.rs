use schemars::JsonSchema;
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::prelude::Expr;
use sea_orm::sea_query::Alias;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, ExprTrait, FromJsonQueryResult, FromQueryResult, QueryFilter, QuerySelect};
use sea_orm::sqlx::types::chrono::{self, DateTime, Local};
use sea_orm::{DbErr};
use serde::Serialize;
use uuid::Uuid;
use crate::auth::get_by_user::AuthGetUuidError;
use crate::entity::{genertic_header, mvp_data, mvp_scouters, scout_game_midway_insert, upcoming_game};
use crate::{SETTINGS, auth, backenddb};
use crate::define_games;
use itertools::Itertools;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::entity::sea_orm_active_enums::{Stations, TournamentLevels};
use crate::snowgrave::datatypes::Team;

/// What a main defender spent the match defending.
///
/// Only meaningful when the game is flagged as the main defender — it is `Some`
/// exactly when that flag is set, and `None` otherwise. `Alliance` means the
/// defence was spread across the whole opposing alliance (the original DPDG
/// behaviour, summed over all three opponents); `Bot` means a single opposing
/// robot was targeted, and DPDG is computed against only that robot's score and
/// event average.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, serde::Deserialize, JsonSchema, FromJsonQueryResult)]
pub enum DefenceTarget {
    Alliance,
    Bot(Team),
}

/// Prescout rows are admin-entered data for matches that were never queued, and
/// they must never be mixed with real scouting data — so every read of
/// `genertic_header` picks a side explicitly.
///
/// Note the three states: `NULL` means the row was written before this flag
/// existed, i.e. normal data. That is why the normal case cannot be expressed as
/// `.ne(true)` — in SQL `NULL <> true` is `NULL`, which is falsy, and every
/// pre-existing row would silently disappear from averages.
pub fn prescout_filter(only_prescout: bool) -> Condition {
    if only_prescout {
        Condition::all().add(genertic_header::Column::IsPrescout.eq(true))
    } else {
        Condition::any()
            .add(genertic_header::Column::IsPrescout.is_null())
            .add(genertic_header::Column::IsPrescout.eq(false))
    }
}

async fn to_full_match(model: genertic_header::Model, db: &DatabaseConnection) -> Result<HeaderFull, DbErr> {
    let mut username_str: Vec<String> = Vec::with_capacity(model.user.len());

    for user in &model.user {
        let user_str =  auth::get_by_user::get_by_uuid(user, db).await.unwrap();
        username_str.push(user_str);
    }

    // Fetch the MVP comment for this game from mvp_data
    let mvp_comment: Option<String> = async {
        let game = upcoming_game::Entity::find()
            .filter(upcoming_game::Column::MatchId.eq(model.match_id))
            .filter(upcoming_game::Column::Set.eq(model.set))
            .filter(upcoming_game::Column::EventCode.eq(&model.event_code))
            .filter(upcoming_game::Column::TournamentLevel.eq(model.tournament_level))
            .one(db)
            .await
            .ok()??;

        let mvp_scouter_id = match model.station {
            Stations::Red1 | Stations::Red2 | Stations::Red3 => game.mvp_id_red?,
            Stations::Blue1 | Stations::Blue2 | Stations::Blue3 => game.mvp_id_blue?,
        };

        let scouter = mvp_scouters::Entity::find_by_id(mvp_scouter_id)
            .one(db)
            .await
            .ok()??;

        let data_id = scouter.data?;

        let data = mvp_data::Entity::find_by_id(data_id)
            .one(db)
            .await
            .ok()??;

        Some(data.comment)
    }.await;

    Ok(HeaderFull {
        id: model.id,
        user: username_str,
        team: model.team,
        is_ab_team: model.is_ab_team,
        match_id: model.match_id,
        set: model.set,
        total_score: model.total_score,
        event_code: model.event_code,
        tournament_level: model.tournament_level,
        station: model.station,
        created_at: model.created_at,
        is_mvp: model.is_mvp,
        defence: model.defence,
        auto_score: model.auto_score,
        comment: model.comment,
        teleop_score: model.teleop_score,
        mvp_comment,
        dpdg: None,
        dpdg_raw: None,
    })
}

async fn to_full_match_midway(model: scout_game_midway_insert::Model, db: &DatabaseConnection) -> Result<HeaderFull, DbErr> {
    let username = match auth::get_by_user::get_by_uuid(&model.user, db).await {
        Ok(u) => u,
        Err(_) => model.user.to_string(),
    };

    Ok(HeaderFull {
        id: model.id,
        user: vec![username],
        team: model.team,
        is_ab_team: model.is_ab_team,
        match_id: model.match_id,
        set: model.set,
        total_score: model.total_score as f32,
        auto_score: model.auto_score as f32,
        teleop_score: model.teleop_score as f32,
        event_code: model.event_code,
        tournament_level: model.tournament_level,
        station: model.station,
        comment: model.comment,
        created_at: model.created_at,
        is_mvp: false,
        defence: model.defence as f32,
        mvp_comment: None,
        dpdg: None,
        dpdg_raw: None,
    })
}

pub struct InsertReturn {
    pub game_type: i32,
    pub game_id: i32,
    pub total_score: f32,
    pub teleop_score: f32,
    pub auto_score: f32,
}

pub struct AvgReturn {
    pub team: i32,
    pub is_ab_team: bool,
    pub data: GamesAvgSpecific
}

pub struct FrontRunnerReturn {
    pub crazy: Vec<usize>,
    pub avg: Option<GamesInserts>
}

pub struct Scores {
    pub total_score: f32,
    pub teleop_score: f32,
    pub auto_score: f32,
}

#[async_trait]
pub trait YearOp: Send + Sync {
    fn get_year_id(&self) -> i32;
    async fn insert(&self, data: &GamesInsertsSpecific, db: &DatabaseConnection) -> Result<InsertReturn, DbErr>;
    // Not is the name order!
    async fn average_team(&self, ids: Vec<TeamGameUnMergedData>, db: &DatabaseConnection) -> Result<Vec<AvgReturn>, DbErr>;
    async fn get_full_matches(&self, game_ids: Vec<i32>, db: &DatabaseConnection) -> Result<Vec<GamesFullSpecific>, DbErr>;
    async fn delete(&self, id: i32, db: &DatabaseConnection) -> Result<(), DbErr>;
    #[allow(dead_code)]
    async fn get(&self, id: i32, db: &DatabaseConnection) -> Result<GamesFullSpecific, DbErr>;
    fn frontrunner_op(&self, games: &FrontRunnerGame) -> Result<FrontRunnerReturn, DbErr>;
    fn get_scores(&self, match_data: &GamesInsertsSpecific) -> Scores;
    async fn edit(&self, game_id: i32, edit: GamesEditSpecific, db: &DatabaseConnection) -> Result<InsertReturn, DbErr>;
    // DPDG is computed during the checking phase (when all opponent data is present)
    // and stamped into the game-specific insert before it is persisted.
    fn main_defender(&self, _game: &GamesInsertsSpecific) -> bool {
        false
    }
    fn set_dpdg(&self, _game: &mut GamesInsertsSpecific, _dpdg: Option<f32>, _dpdg_raw: Option<f32>) {
    }
    /// What this game's main defender was defending, or `None` if it isn't one.
    fn defence_target(&self, _game: &GamesInsertsSpecific) -> Option<DefenceTarget> {
        None
    }
}

/// Extract the stored defence target (if any) from a fully loaded game.
pub fn defence_target_from_game(game: &GamesFullSpecific) -> Option<DefenceTarget> {
    let GamesFullSpecific::RebuiltGame(model) = game;
    if !model.defence_main {
        return None;
    }
    Some(model.defence_target.unwrap_or(DefenceTarget::Alliance))
}

/// Extract the stored DPDG values (if any) from a fully loaded game.
/// Returns `(percentage, raw points)`.
pub fn dpdg_from_game(game: &GamesFullSpecific) -> (Option<f32>, Option<f32>) {
    let GamesFullSpecific::RebuiltGame(model) = game;
    (model.dpdg, model.dpdg_raw)
}


//A common header that will be used for Insert data
#[derive(Debug)]
pub struct HeaderInsert {
    //Id is given by server
    pub user: Vec<Uuid>, //We will get uuid
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    //Total score is irraiven as it will be computed at server side
    pub defence: f32,
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    pub is_mvp: bool,
    pub comment: String,
    /// Admin-entered prescout data — excluded from every normal calculation.
    pub is_prescout: bool,
    //Created At no need to import as this will be seen by the server
    //game_type_id polymorfism will be seen by the enum
    //No need for game id as that will be seen by the enum
}

async fn prim_insert_game(data: &GamesInserts, model: Box<dyn YearOp>, db: &DatabaseConnection) -> Result<i32, DbErr> {
    //Insert game spcific
    let res = model.insert(&data.game, db).await?;
    
    let created_at: DateTime<Local> = chrono::Local::now();


    let header_db: genertic_header::ActiveModel = genertic_header::ActiveModel {
        id: NotSet, //Done by db
        user: Set(data.header.user.clone()),
        team: Set(data.header.team),
        is_ab_team: Set(data.header.is_ab_team),
        match_id: Set(data.header.match_id),
        set: Set(data.header.set),
        total_score: Set(res.total_score),
        event_code: Set(data.header.event_code.clone()),
        tournament_level: Set(data.header.tournament_level),
        station: Set(data.header.station),
        created_at: Set(created_at),
        is_mvp: Set(data.header.is_mvp),
        game_type_id: Set(res.game_type),
        game_id: Set(res.game_id),
        teleop_score: Set(res.teleop_score),
        auto_score: Set(res.auto_score),
        defence: Set(data.header.defence),
        comment: Set(data.header.comment.clone()),
        is_prescout: Set(Some(data.header.is_prescout)),
    };
    Ok(genertic_header::Entity::insert(header_db).exec(db).await?.last_insert_id)
}



async fn prim_search_game(mode: Box<dyn YearOp>, param: &SearchParam, db: &DatabaseConnection) -> Result<Vec<GamesFull>, DbErr> {
    let prescout_only = param.prescout_only == Some(true);

    let mut game_headers = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(param.year))
        .filter(prescout_filter(prescout_only));

    if let Some(user) = &param.user {
        let a = match crate::auth::get_by_user::get_by_username(user, db).await {
            Ok(a) => a,
            Err(a) => {
                match a {
                    AuthGetUuidError::UserIsNotHere => {
                        return Err(DbErr::Custom("User was not found".to_string()));
                    },
                    AuthGetUuidError::DatabaseError(db_err) => {
                        return Err(db_err);
                    },
                }
            },
        };    
        game_headers = game_headers.filter(
            //todo: make this more proper
            Expr::cust(format!("'{}' = ANY(\"user\")", a))
        );
    }
    if let Some(teams) = &param.teams {
        if !teams.is_empty() {
            let mut cond = Condition::any();
            for t in teams {
                cond = cond.add(
                    Condition::all()
                        .add(genertic_header::Column::Team.eq(t.number))
                        .add(genertic_header::Column::IsAbTeam.eq(t.is_ab_team))
                );
            }
            game_headers = game_headers.filter(cond);
        }
    }
    if let Some(match_id) = &param.match_id {
        game_headers = game_headers.filter(genertic_header::Column::MatchId.eq(*match_id));
    }
    if let Some(set) = &param.set {
        game_headers = game_headers.filter(genertic_header::Column::Set.eq(*set));
    }
    if let Some(total_score) = &param.total_score {
        game_headers = game_headers.filter(genertic_header::Column::TotalScore.eq(*total_score));
    }
    if let Some(event_code) = &param.event_code {
        game_headers = game_headers.filter(genertic_header::Column::EventCode.eq(event_code));
    }
    if let Some(tournament_level) = &param.tournament_level {
        game_headers = game_headers.filter(genertic_header::Column::TournamentLevel.eq(*tournament_level));
    }
    if let Some(station) = &param.station {
        game_headers = game_headers.filter(genertic_header::Column::Station.eq(*station));
    }
    if let Some(mvp) = &param.is_mvp {
        game_headers = game_headers.filter(genertic_header::Column::IsMvp.eq(*mvp));
    }

    let res = game_headers.all(db).await?;
    let ids: Vec<i32> = res.iter().map(|a| a.game_id).collect();

    let mut header: Vec<HeaderFull> = Vec::with_capacity(res.len());
    for head in res {
        header.push(to_full_match(head, db).await?);
    }


    let games = mode.get_full_matches(ids, db).await?;

    let mut merged: Vec<GamesFull> = header.into_iter().zip(games.into_iter())
        .map(|x| GamesFull {header: x.0, game: x.1}).collect();

    // Midway rows live in `mid_header` and can never be prescout, so folding
    // them in during a prescout-only search would be exactly the mixing this
    // flag exists to prevent.
    if param.include_midway == Some(true) && !prescout_only {
        let mut mid_query = scout_game_midway_insert::Entity::find()
            .filter(scout_game_midway_insert::Column::GameTypeId.eq(param.year));

        if let Some(user) = &param.user {
            let uuid = match crate::auth::get_by_user::get_by_username(user, db).await {
                Ok(u) => u,
                Err(AuthGetUuidError::UserIsNotHere) => return Err(DbErr::Custom("User was not found".to_string())),
                Err(AuthGetUuidError::DatabaseError(e)) => return Err(e),
            };
            mid_query = mid_query.filter(scout_game_midway_insert::Column::User.eq(uuid));
        }
        if let Some(teams) = &param.teams {
            if !teams.is_empty() {
                let mut cond = Condition::any();
                for t in teams {
                    cond = cond.add(
                        Condition::all()
                            .add(scout_game_midway_insert::Column::Team.eq(t.number))
                            .add(scout_game_midway_insert::Column::IsAbTeam.eq(t.is_ab_team))
                    );
                }
                mid_query = mid_query.filter(cond);
            }
        }
        if let Some(match_id) = &param.match_id {
            mid_query = mid_query.filter(scout_game_midway_insert::Column::MatchId.eq(*match_id));
        }
        if let Some(set) = &param.set {
            mid_query = mid_query.filter(scout_game_midway_insert::Column::Set.eq(*set));
        }
        if let Some(event_code) = &param.event_code {
            mid_query = mid_query.filter(scout_game_midway_insert::Column::EventCode.eq(event_code));
        }
        if let Some(tournament_level) = &param.tournament_level {
            mid_query = mid_query.filter(scout_game_midway_insert::Column::TournamentLevel.eq(*tournament_level));
        }
        if let Some(station) = &param.station {
            mid_query = mid_query.filter(scout_game_midway_insert::Column::Station.eq(*station));
        }

        // Only include midway records that haven't been finalized yet
        let all_mid = mid_query.all(db).await?;
        let mid_res: Vec<_> = all_mid
            .into_iter()
            .filter(|m| !merged.iter().any(|g|
                g.header.team == m.team &&
                g.header.is_ab_team == m.is_ab_team &&
                g.header.match_id == m.match_id &&
                g.header.set == m.set &&
                g.header.event_code == m.event_code &&
                g.header.tournament_level == m.tournament_level &&
                g.header.station == m.station
            ))
            .collect();

        let mid_game_ids: Vec<i32> = mid_res.iter().map(|m| m.game_id).collect();
        let mid_games = mode.get_full_matches(mid_game_ids, db).await?;

        let mut mid_headers: Vec<HeaderFull> = Vec::with_capacity(mid_res.len());
        for mid in mid_res {
            mid_headers.push(to_full_match_midway(mid, db).await?);
        }

        let mid_merged: Vec<GamesFull> = mid_headers.into_iter().zip(mid_games.into_iter())
            .map(|(h, g)| GamesFull { header: h, game: g })
            .collect();
        merged.extend(mid_merged);
    }

    for full in merged.iter_mut() {
        let (dpdg, dpdg_raw) = dpdg_from_game(&full.game);
        full.header.dpdg = dpdg;
        full.header.dpdg_raw = dpdg_raw;
    }

    Ok(merged)
}


#[derive(FromQueryResult)]
struct NormalGenDataAvg {
    pub team: i32,
    pub is_ab_team: bool,
    pub total_score: f64,
    pub auto_score: f64,
    pub teleop_score: f64,
    pub defence_score: f64,
    pub mvp_percent: f64,
    pub record_count: i64,
}

#[derive(FromQueryResult)]
struct MidDataAvg {
    pub team: i32,
    pub is_ab_team: bool,
    pub total_score: f64,
    pub auto_score: f64,
    pub teleop_score: f64,
    pub defence_score: f64,
    pub record_count: i64,
}

#[derive(FromQueryResult)]
struct FinalizedMatchKey {
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
}

#[derive(FromQueryResult)]
struct MidSpcDataAvg {
    pub team: i32,
    pub is_ab_team: bool,
    pub game_id: i32,
    pub match_id: i32,
    pub set: i32,
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
}
#[derive(FromQueryResult)]
pub struct NormalSpcDataAvg {
    pub team: i32,
    pub is_ab_team: bool,
    pub game_id: i32,
}

pub struct TeamGameUnMergedData {
    pub team: i32,
    pub is_ab_team: bool,
    pub game_ids: Vec<i32>,
}

pub struct Scores_E {
    pub total_score: f64,
    pub auto_score: f64,
    pub teleop_score: f64,
    pub defence: f64,
    pub mvp_percent: f64,
}

async fn prim_average_game(model: Box<dyn YearOp>, event_code: &String, include_midway: bool, db: &DatabaseConnection) -> Result<Vec<TeamAvg>, DbErr> {
    let select_avg_score: Vec<NormalGenDataAvg> = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()))
        .filter(genertic_header::Column::EventCode.eq(event_code))
        .filter(prescout_filter(false))
        .select_only()
        .column_as(genertic_header::Column::TotalScore.avg().cast_as(Alias::new("FLOAT8")), "total_score")
        .column_as(genertic_header::Column::AutoScore.avg().cast_as(Alias::new("FLOAT8")), "auto_score")
        .column_as(genertic_header::Column::TeleopScore.avg().cast_as(Alias::new("FLOAT8")), "teleop_score")
        .column_as(genertic_header::Column::Defence.avg().cast_as(Alias::new("FLOAT8")), "defence_score")
        .column_as(Expr::col(genertic_header::Column::IsMvp).cast_as(Alias::new("int")).avg().cast_as(Alias::new("FLOAT8")),"mvp_percent")
        .column_as(genertic_header::Column::Team, "team")
        .column_as(genertic_header::Column::IsAbTeam, "is_ab_team")
        .column_as(Expr::col(genertic_header::Column::Id).count().cast_as(Alias::new("BIGINT")), "record_count")
        .group_by(genertic_header::Column::Team)
        .group_by(genertic_header::Column::IsAbTeam)
        .into_model::<NormalGenDataAvg>()
        .all(db).await?;


    let ids: Vec<NormalSpcDataAvg> = genertic_header::Entity::find()
        .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()))
        .filter(genertic_header::Column::EventCode.eq(event_code))
        .filter(prescout_filter(false))
        .select_only()
        .column(genertic_header::Column::GameId) //Not snowgrave
        .column(genertic_header::Column::Team)
        .column(genertic_header::Column::IsAbTeam)
        .into_model::<NormalSpcDataAvg>().all(db).await?;

    let mut data: HashMap<(i32, bool), Vec<i32>> = ids.into_iter()
        .into_group_map_by(|record| (record.team, record.is_ab_team))
        .into_iter()
        .map(|(key, records)| (key, records.into_iter().map(|r| r.game_id).collect()))
        .collect();

    let mut gen_count_map: HashMap<(i32, bool), i64> = select_avg_score.iter()
        .map(|x| ((x.team, x.is_ab_team), x.record_count))
        .collect();

    let mut avg_map: HashMap<(i32, bool), Scores_E> = select_avg_score.into_iter().map(|x|
        ((x.team, x.is_ab_team), Scores_E {
            total_score: x.total_score,
            auto_score: x.auto_score,
            teleop_score: x.teleop_score,
            defence: x.defence_score,
            mvp_percent: x.mvp_percent,
        })).collect();

    if include_midway {
        // Fetch finalized (team, match, set, event, level, station) keys to avoid double-counting
        let finalized_keys: Vec<FinalizedMatchKey> = genertic_header::Entity::find()
            .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()))
            .filter(genertic_header::Column::EventCode.eq(event_code))
            .filter(prescout_filter(false))
            .select_only()
            .column(genertic_header::Column::Team)
            .column(genertic_header::Column::IsAbTeam)
            .column(genertic_header::Column::MatchId)
            .column(genertic_header::Column::Set)
            .column(genertic_header::Column::EventCode)
            .column(genertic_header::Column::TournamentLevel)
            .column(genertic_header::Column::Station)
            .into_model::<FinalizedMatchKey>()
            .all(db).await?;

        // Fetch per-record midway data with match identity fields for deduplication
        let all_mid_ids: Vec<MidSpcDataAvg> = scout_game_midway_insert::Entity::find()
            .filter(scout_game_midway_insert::Column::GameTypeId.eq(model.get_year_id()))
            .filter(scout_game_midway_insert::Column::EventCode.eq(event_code))
            .select_only()
            .column(scout_game_midway_insert::Column::GameId)
            .column(scout_game_midway_insert::Column::Team)
            .column(scout_game_midway_insert::Column::IsAbTeam)
            .column(scout_game_midway_insert::Column::MatchId)
            .column(scout_game_midway_insert::Column::Set)
            .column(scout_game_midway_insert::Column::EventCode)
            .column(scout_game_midway_insert::Column::TournamentLevel)
            .column(scout_game_midway_insert::Column::Station)
            .into_model::<MidSpcDataAvg>().all(db).await?;

        // Only include midway records with no finalized counterpart
        let mid_ids: Vec<MidSpcDataAvg> = all_mid_ids
            .into_iter()
            .filter(|m| !finalized_keys.iter().any(|f|
                f.team == m.team &&
                f.is_ab_team == m.is_ab_team &&
                f.match_id == m.match_id &&
                f.set == m.set &&
                f.event_code == m.event_code &&
                f.tournament_level == m.tournament_level &&
                f.station == m.station
            ))
            .collect();

        // Build per-team averages from only the unfinalized midway records.
        // mid_header stores scores directly so we can aggregate them in-process.
        let unfinalized_game_ids: std::collections::HashSet<i32> = mid_ids.iter().map(|m| m.game_id).collect();
        let mid_avg_score: Vec<MidDataAvg> = if unfinalized_game_ids.is_empty() {
            vec![]
        } else {
            let mid_raw: Vec<scout_game_midway_insert::Model> = scout_game_midway_insert::Entity::find()
                .filter(scout_game_midway_insert::Column::GameTypeId.eq(model.get_year_id()))
                .filter(scout_game_midway_insert::Column::EventCode.eq(event_code))
                .all(db).await?;

            let mut team_sums: HashMap<(i32, bool), (f64, f64, f64, f64, i64)> = HashMap::new();
            for raw in mid_raw {
                if unfinalized_game_ids.contains(&raw.game_id) {
                    let entry = team_sums.entry((raw.team, raw.is_ab_team)).or_insert((0.0, 0.0, 0.0, 0.0, 0));
                    entry.0 += raw.total_score as f64;
                    entry.1 += raw.auto_score as f64;
                    entry.2 += raw.teleop_score as f64;
                    entry.3 += raw.defence as f64;
                    entry.4 += 1;
                }
            }
            team_sums.into_iter().map(|((team, is_ab_team), (ts, aut, tel, def, cnt))| MidDataAvg {
                team,
                is_ab_team,
                total_score: ts / cnt as f64,
                auto_score: aut / cnt as f64,
                teleop_score: tel / cnt as f64,
                defence_score: def / cnt as f64,
                record_count: cnt,
            }).collect()
        };

        // Merge game_ids from unfinalized midway records into data map
        for mid_id in mid_ids {
            data.entry((mid_id.team, mid_id.is_ab_team))
                .or_insert_with(Vec::new)
                .push(mid_id.game_id);
        }

        // Blend midway header-level averages with existing avg_map
        for mid in mid_avg_score {
            let key = (mid.team, mid.is_ab_team);
            let mid_count = mid.record_count as f64;
            match avg_map.get(&key) {
                Some(existing) => {
                    let gen_count = *gen_count_map.get(&key).unwrap_or(&0) as f64;
                    let total = gen_count + mid_count;
                    let blended = Scores_E {
                        total_score: (existing.total_score * gen_count + mid.total_score * mid_count) / total,
                        auto_score: (existing.auto_score * gen_count + mid.auto_score * mid_count) / total,
                        teleop_score: (existing.teleop_score * gen_count + mid.teleop_score * mid_count) / total,
                        defence: (existing.defence * gen_count + mid.defence_score * mid_count) / total,
                        mvp_percent: existing.mvp_percent,
                    };
                    avg_map.insert(key, blended);
                },
                None => {
                    // Team only has midway data
                    avg_map.insert(key, Scores_E {
                        total_score: mid.total_score,
                        auto_score: mid.auto_score,
                        teleop_score: mid.teleop_score,
                        defence: mid.defence_score,
                        mvp_percent: 0.0,
                    });
                }
            }
        }
    }

    let data_vec: Vec<TeamGameUnMergedData> = data.into_iter()
        .map(|((team, is_ab_team), game_ids)| TeamGameUnMergedData { team, is_ab_team, game_ids })
        .collect();

    let a: Vec<AvgReturn> = model.average_team(data_vec, db).await?;
    let mut done: Vec<TeamAvg> = Vec::with_capacity(a.len());
    for team_avg_data in a {
        let avg = avg_map.get(&(team_avg_data.team, team_avg_data.is_ab_team)).ok_or(DbErr::AttrNotSet("Could not find avg data".to_string()))?;
        let GamesAvgSpecific::RebuiltGame(avg_data) = &team_avg_data.data;
        let dpdg = avg_data.dpdg_avg;
        let dpdg_raw = avg_data.dpdg_raw_avg;
        done.push(TeamAvg { team: team_avg_data.team,
            is_ab_team: team_avg_data.is_ab_team,
            total_score: avg.total_score,
            auto_score: avg.auto_score,
            teleop_score: avg.teleop_score,
            defence_score: avg.defence,
            game: team_avg_data.data,
            mvp_percent: avg.mvp_percent,
            dpdg,
            dpdg_raw,
        });
    }



    Ok(done)
}
#[allow(dead_code)]
async fn prim_get_game(model: Box<dyn YearOp>, id: i32, db: &DatabaseConnection) -> Result<GamesFull, DbErr> {
    let header = match genertic_header::Entity::find_by_id(id).one(db).await? {
        None => {
            return Err(DbErr::RecordNotFound("Could not find".to_string()));
        },
        Some(a) => {
            a
        }
    };

    let game = model.get(header.game_id, db).await?;
    let right_header = to_full_match(header, db).await?;

    Ok(GamesFull {
        header: right_header,
        game,
    })
}


#[derive(Debug)]
pub struct GamesInserts {
    pub header: HeaderInsert,
    pub game: GamesInsertsSpecific
}


#[derive(Serialize, JsonSchema, Deserialize, Debug)]
pub struct GamesGraph {
    pub time: DateTime<Local>,
    pub total_score: f32,
    pub auto_score: f32,
    pub teleop_score: f32,
    pub defence: f32,
    pub dpdg: Option<f32>,
    pub dpdg_raw: Option<f32>,
}

#[derive(FromQueryResult)]
struct GamesGraphRow {
    pub game_id: i32,
    pub time: DateTime<Local>,
    pub total_score: f32,
    pub auto_score: f32,
    pub teleop_score: f32,
    pub defence: f32,
}

#[derive(Serialize, JsonSchema)]
pub struct TeamAvg {
    pub team: i32,
    pub is_ab_team: bool,
    pub total_score: f64,
    pub auto_score: f64,
    pub teleop_score: f64,
    pub defence_score: f64,
    pub mvp_percent: f64,
    pub dpdg: Option<f64>,
    pub dpdg_raw: Option<f64>,
    pub game: GamesAvgSpecific
}

#[derive(Serialize, JsonSchema)]
pub struct GamesFull {
    pub header: HeaderFull,
    pub game: GamesFullSpecific
}

pub struct GamesEdit {
    pub header: HeaderFullEdit,
    pub game: GamesEditSpecific
}

pub struct SearchParam {
    //Id should be done via get
    pub user: Option<String>,
    pub teams: Option<Vec<Team>>,
    pub match_id: Option<i32>,
    pub set: Option<i32>,
    pub total_score: Option<i32>,
    pub event_code: Option<String>,
    pub tournament_level: Option<TournamentLevels>,
    pub is_mvp: Option<bool>,
    pub station: Option<Stations>,
    pub year: i32,
    pub include_midway: Option<bool>,
    /// Return prescout rows *and only* prescout rows. Prescout data is never
    /// mixed in with real scouting data, so this is exclusive rather than
    /// additive. Defaults to normal (non-prescout) results.
    pub prescout_only: Option<bool>,
}

#[derive(Serialize, JsonSchema)]
pub struct HeaderFull {
    pub id: i32,
    pub user: Vec<String>,
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    pub total_score: f32,
    pub auto_score: f32,
    pub teleop_score: f32,
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    pub comment: String,
    pub created_at: DateTime<Local>,
    pub is_mvp: bool,
    pub defence: f32,
    pub mvp_comment: Option<String>,
    /// DPDG as a percentage of the opposing teams' event averages.
    pub dpdg: Option<f32>,
    /// DPDG as a raw point value.
    pub dpdg_raw: Option<f32>,
}

#[derive()]
pub struct FrontRunnerGame {
    pub games: Vec<(Uuid, GamesFullSpecific)>,
    pub defence: Vec<f32>,
    pub comment: Vec<String>,
    pub team: i32,
    pub is_ab_team: bool,
    pub match_id: i32,
    pub set: i32,
    //Total score is irraiven as it will be computed at server side
    pub event_code: String,
    pub tournament_level: TournamentLevels,
    pub station: Stations,
    pub is_mvp: bool,
}

pub struct HeaderFullEdit {
    pub id: i32,
    pub user: Option<Vec<Uuid>>,
    pub team: Option<i32>,
    pub is_ab_team: Option<bool>,
    pub match_id: Option<i32>,
    pub set: Option<i32>,
    pub event_code: Option<String>,
    pub tournament_level: Option<TournamentLevels>,
    pub station: Option<Stations>,
    pub created_at: Option<DateTime<Local>>,
    pub is_mvp: Option<bool>,
    pub defence: Option<f32>,
    pub comment: Option<String>
}

async fn to_full_am(header: HeaderFullEdit, db: &DatabaseConnection) -> Result<genertic_header::ActiveModel, DbErr> {
    //get gametype and game id (for later insert into game)
    let game_model = match genertic_header::Entity::find_by_id(header.id).one(db).await? {
        Some(a) => a,
        None => {
            return Err(DbErr::Custom("Not a vaild ID".to_string()));
        },
    };
    
    Ok(genertic_header::ActiveModel {
        id: NotSet,
        team: header.team.map(Set).unwrap_or(NotSet),
        is_ab_team: header.is_ab_team.map(Set).unwrap_or(NotSet),
        match_id: header.match_id.map(Set).unwrap_or(NotSet),
        set: header.set.map(Set).unwrap_or(NotSet),
        total_score: NotSet, //Will be set later
        event_code: header.event_code.map(Set).unwrap_or(NotSet),
        tournament_level: header.tournament_level.map(Set).unwrap_or(NotSet),
        station: header.station.map(Set).unwrap_or(NotSet),
        created_at: header.created_at.map(Set).unwrap_or(NotSet),
        is_mvp: header.is_mvp.map(Set).unwrap_or(NotSet),
        game_type_id: Set(game_model.game_type_id),
        game_id: Set(game_model.game_id),
        // Carried forward from the stored row — an edit never reclassifies a
        // game between prescout and real scouting data.
        is_prescout: Set(game_model.is_prescout),
        teleop_score: NotSet,
        auto_score: NotSet,
        defence: header.defence.map(Set).unwrap_or(NotSet),
        comment: header.comment.map(Set).unwrap_or(NotSet),
        user: header.user.map(Set).unwrap_or(NotSet),
    })
}

//REAL functions that help grab the value

pub async fn insert_game(data: &GamesInserts, db: &DatabaseConnection) -> Result<i32, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_insert_game(data, game, db).await
}

pub async fn graph_game(team: &i32, is_ab_team: &bool, event_code: &Option<String>, db: &DatabaseConnection) -> Result<Vec<GamesGraph>, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    let mut command = genertic_header::Entity::find()
        .filter(genertic_header::Column::Team.eq(*team))
        .filter(genertic_header::Column::IsAbTeam.eq(*is_ab_team))
        .filter(genertic_header::Column::GameTypeId.eq(game.get_year_id()))
        .filter(prescout_filter(false));
    if let Some(e) = event_code {
        command = command.filter(genertic_header::Column::EventCode.eq(e));
    }
    let rows: Vec<GamesGraphRow> = command
        .select_only()
        .column(genertic_header::Column::GameId)
        .column_as(genertic_header::Column::CreatedAt, "time")
        .column_as(genertic_header::Column::TotalScore, "total_score")
        .column_as(genertic_header::Column::AutoScore, "auto_score")
        .column_as(genertic_header::Column::TeleopScore, "teleop_score")
        .column_as(genertic_header::Column::Defence, "defence")
        .into_model::<GamesGraphRow>()
        .all(db)
    .await?;

    let game_ids: Vec<i32> = rows.iter().map(|r| r.game_id).collect();
    let games = game.get_full_matches(game_ids, db).await?;
    let dpdg_map: HashMap<i32, (Option<f32>, Option<f32>)> = games.into_iter().map(|g| {
        let GamesFullSpecific::RebuiltGame(model) = &g;
        (model.id, (model.dpdg, model.dpdg_raw))
    }).collect();

    let res: Vec<GamesGraph> = rows.into_iter().map(|r| GamesGraph {
        time: r.time,
        total_score: r.total_score,
        auto_score: r.auto_score,
        teleop_score: r.teleop_score,
        defence: r.defence,
        dpdg: dpdg_map.get(&r.game_id).and_then(|(pct, _)| *pct),
        dpdg_raw: dpdg_map.get(&r.game_id).and_then(|(_, raw)| *raw),
    }).collect();

    Ok(res)
}

pub async fn search_game(param: &SearchParam, db: &DatabaseConnection) -> Result<Vec<GamesFull>, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_search_game(game, param, db).await
}

pub async fn average_game(event_code: &String, include_midway: bool, db: &DatabaseConnection) -> Result<Vec<TeamAvg>, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_average_game(game, event_code, include_midway, db).await
}

pub async fn frontrunner(games: &FrontRunnerGame) -> Result<FrontRunnerReturn, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    game.frontrunner_op(games)
}

#[allow(dead_code)]
pub async fn get_game(id: i32, db: &DatabaseConnection) -> Result<GamesFull, DbErr> {
    let game = game_dispatch(SETTINGS.year);

    prim_get_game(game, id, db).await
}

pub async fn delete_game(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    let game = game_dispatch(SETTINGS.year);

    game.delete(id, db).await
}


define_games!(
    //Insert each year here
    RebuiltGame => crate::backenddb::entrys::rebuilt
);  
pub mod game {
    use sea_orm::ActiveValue::{NotSet, Set};
    use sea_orm::dynamic::Column;
    use sea_orm::sea_query::SelectStatement;
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult,
        QueryFilter, QuerySelect,
    };
    use sea_orm::sqlx::types::chrono::{self, DateTime, Local, TimeZone};
    use sea_orm::DbErr;
    use serde::Serialize;
    use serde_json::Value;
    use uuid::Uuid;
    use crate::auth::get_by_user::{AuthGetUuidError, get_by_uuid};
    use crate::backenddb::example_game::Avg;
    use crate::entity::genertic_header;
    use crate::entity::prelude::GenerticHeader;
    use crate::{SETTINGS, auth, backenddb::*};
    use crate::define_games;
    use itertools::Itertools;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use crate::entity::sea_orm_active_enums::{Stations, TournamentLevels};
    async fn to_full_match(
        model: genertic_header::Model,
        db: &DatabaseConnection,
    ) -> Result<HeaderFull, DbErr> {
        let created_at = match Local.from_local_datetime(&model.created_at).single() {
            Some(a) => a,
            None => {
                return Err(DbErr::Custom("Could not parse time!".to_string()));
            }
        };
        let username = match auth::get_by_user::get_by_uuid(&model.user, db).await {
            Ok(a) => a,
            Err(a) => {
                match a {
                    AuthGetUuidError::UserIsNotHere => {
                        return Err(DbErr::Custom("User was not found".to_string()));
                    }
                    AuthGetUuidError::DatabaseError(db_err) => {
                        return Err(db_err);
                    }
                }
            }
        };
        Ok(HeaderFull {
            id: model.id,
            user: username,
            team: model.team,
            is_ab_team: model.is_ab_team,
            match_id: model.match_id,
            set: model.set,
            total_score: model.total_score,
            event_code: model.event_code,
            tournament_level: model.tournament_level,
            station: model.station,
            created_at: created_at,
        })
    }
    pub trait YearOp: Send + Sync {
        fn get_year_id(&self) -> i32;
        #[must_use]
        #[allow(
            elided_named_lifetimes,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds
        )]
        fn insert<'life0, 'life1, 'life2, 'async_trait>(
            &'life0 self,
            data: &'life1 GamesInsertsSpecific,
            db: &'life2 DatabaseConnection,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = Result<(i32, i32, i32), DbErr>,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            'life2: 'async_trait,
            Self: 'async_trait;
        #[must_use]
        #[allow(
            elided_named_lifetimes,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds
        )]
        fn graph<'life0, 'life1, 'async_trait>(
            &'life0 self,
            ids: Vec<i32>,
            db: &'life1 DatabaseConnection,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = Result<Vec<GamesGraphSpecific>, DbErr>,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait;
        #[must_use]
        #[allow(
            elided_named_lifetimes,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds
        )]
        fn average_team<'life0, 'life1, 'async_trait>(
            &'life0 self,
            ids: Vec<(i32, Vec<i32>)>,
            db: &'life1 DatabaseConnection,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = Result<Vec<(i32, GamesAvgSpecific)>, DbErr>,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait;
        #[must_use]
        #[allow(
            elided_named_lifetimes,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds
        )]
        fn get_full_matches<'life0, 'life1, 'async_trait>(
            &'life0 self,
            ids: Vec<i32>,
            db: &'life1 DatabaseConnection,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = Result<Vec<GamesFullSpecific>, DbErr>,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait;
        #[must_use]
        #[allow(
            elided_named_lifetimes,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds
        )]
        fn delete<'life0, 'life1, 'async_trait>(
            &'life0 self,
            id: i32,
            db: &'life1 DatabaseConnection,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = Result<(), DbErr>,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait;
        #[must_use]
        #[allow(
            elided_named_lifetimes,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds
        )]
        fn get<'life0, 'life1, 'async_trait>(
            &'life0 self,
            id: i32,
            db: &'life1 DatabaseConnection,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = Result<GamesFullSpecific, DbErr>,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait;
        #[must_use]
        #[allow(
            elided_named_lifetimes,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds
        )]
        fn edit<'life0, 'life1, 'async_trait>(
            &'life0 self,
            id: i32,
            edit: GamesEditSpecific,
            db: &'life1 DatabaseConnection,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = Result<(), DbErr>,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait;
    }
    pub struct HeaderInsert {
        pub user: String,
        pub team: i32,
        pub is_ab_team: bool,
        pub match_id: i32,
        pub set: i32,
        pub event_code: String,
        pub tournament_level: TournamentLevels,
        pub station: Stations,
    }
    async fn prim_insert_game(
        data: &GamesInserts,
        model: Box<dyn YearOp>,
        db: &DatabaseConnection,
    ) -> Result<i32, DbErr> {
        let (game_type_id, game_id, total_score) = model.insert(&data.game, db).await?;
        let a = match crate::auth::get_by_user::get_by_username(&data.header.user, db)
            .await
        {
            Ok(a) => a,
            Err(a) => {
                match a {
                    AuthGetUuidError::UserIsNotHere => {
                        return Err(DbErr::Custom("User was not found".to_string()));
                    }
                    AuthGetUuidError::DatabaseError(db_err) => {
                        return Err(db_err);
                    }
                }
            }
        };
        let created_at: DateTime<Local> = chrono::Local::now();
        let header_db: genertic_header::ActiveModel = genertic_header::ActiveModel {
            id: NotSet,
            user: Set(a),
            team: Set(data.header.team),
            is_ab_team: Set(data.header.is_ab_team),
            match_id: Set(data.header.match_id),
            set: Set(data.header.set),
            total_score: Set(total_score),
            event_code: Set(data.header.event_code.clone()),
            tournament_level: Set(data.header.tournament_level.clone()),
            station: Set(data.header.station.clone()),
            created_at: Set(created_at.naive_local()),
            game_type_id: Set(game_type_id),
            game_id: Set(game_id),
        };
        Ok(genertic_header::Entity::insert(header_db).exec(db).await?.last_insert_id)
    }
    async fn prim_graph_game(
        model: Box<dyn YearOp>,
        team: &i32,
        event_code: &Option<String>,
        db: &DatabaseConnection,
    ) -> Result<Vec<GamesGraph>, DbErr> {
        let mut command = genertic_header::Entity::find()
            .filter(genertic_header::Column::Team.eq(*team))
            .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()));
        if let Some(e) = event_code {
            command = command.filter(genertic_header::Column::EventCode.eq(e));
        }
        let res: Vec<(HeaderGraph, i32)> = command
            .select_only()
            .column(genertic_header::Column::CreatedAt)
            .column(genertic_header::Column::TotalScore)
            .column(genertic_header::Column::GameId)
            .into_tuple()
            .all(db)
            .await?
            .iter()
            .map(|x: &(DateTime<Local>, i32, i32)| {
                (
                    HeaderGraph {
                        time: x.0,
                        total_score: x.1,
                    },
                    x.2,
                )
            })
            .collect();
        let game_data = model.graph(res.iter().map(|x| x.1).collect(), db).await?;
        let header: Vec<HeaderGraph> = res.into_iter().map(|x| x.0).collect();
        let merged: Vec<GamesGraph> = header
            .into_iter()
            .zip(game_data.into_iter())
            .map(|(header, game)| GamesGraph { header, game })
            .collect();
        Ok(merged)
    }
    async fn prim_search_game(
        mode: Box<dyn YearOp>,
        param: &SearchParam,
        db: &DatabaseConnection,
    ) -> Result<Vec<GamesFull>, DbErr> {
        let mut game_headers = genertic_header::Entity::find()
            .filter(genertic_header::Column::GameTypeId.eq(param.year));
        if let Some(user) = &param.user {
            let a = match crate::auth::get_by_user::get_by_username(user, db).await {
                Ok(a) => a,
                Err(a) => {
                    match a {
                        AuthGetUuidError::UserIsNotHere => {
                            return Err(DbErr::Custom("User was not found".to_string()));
                        }
                        AuthGetUuidError::DatabaseError(db_err) => {
                            return Err(db_err);
                        }
                    }
                }
            };
            game_headers = game_headers.filter(genertic_header::Column::User.eq(a));
        }
        if let Some(team) = &param.team {
            game_headers = game_headers.filter(genertic_header::Column::Team.eq(*team));
        }
        if let Some(is_ab_team) = &param.is_ab_team {
            game_headers = game_headers
                .filter(genertic_header::Column::IsAbTeam.eq(*is_ab_team));
        }
        if let Some(match_id) = &param.match_id {
            game_headers = game_headers
                .filter(genertic_header::Column::MatchId.eq(*match_id));
        }
        if let Some(set) = &param.set {
            game_headers = game_headers.filter(genertic_header::Column::Set.eq(*set));
        }
        if let Some(total_score) = &param.total_score {
            game_headers = game_headers
                .filter(genertic_header::Column::TotalScore.eq(*total_score));
        }
        if let Some(event_code) = &param.event_code {
            game_headers = game_headers
                .filter(genertic_header::Column::EventCode.eq(event_code));
        }
        if let Some(tournament_level) = &param.tournament_level {
            game_headers = game_headers
                .filter(
                    genertic_header::Column::TournamentLevel.eq(tournament_level.clone()),
                );
        }
        if let Some(station) = &param.station {
            game_headers = game_headers
                .filter(genertic_header::Column::EventCode.eq(station.clone()));
        }
        let res = game_headers.all(db).await?;
        let ids: Vec<i32> = res.iter().map(|a| a.game_id).collect();
        let mut header: Vec<HeaderFull> = Vec::with_capacity(res.len());
        for head in res {
            header.push(to_full_match(head, db).await?);
        }
        let games = mode.get_full_matches(ids, db).await?;
        let merged: Vec<GamesFull> = header
            .into_iter()
            .zip(games.into_iter())
            .map(|x| GamesFull {
                header: x.0,
                game: x.1,
            })
            .collect();
        Ok(merged)
    }
    struct NormalGenDataAvg {
        pub team: i32,
        pub total_score: f32,
    }
    #[automatically_derived]
    impl sea_orm::FromQueryResult for NormalGenDataAvg {
        fn from_query_result(
            row: &sea_orm::QueryResult,
            pre: &str,
        ) -> std::result::Result<Self, sea_orm::DbErr> {
            Ok(Self::from_query_result_nullable(row, pre)?)
        }
        fn from_query_result_nullable(
            row: &sea_orm::QueryResult,
            pre: &str,
        ) -> std::result::Result<Self, sea_orm::TryGetError> {
            let team = match row.try_get_nullable(pre, "team") {
                Err(v @ sea_orm::TryGetError::DbErr(_)) => {
                    return Err(v);
                }
                v => v,
            };
            let total_score = match row.try_get_nullable(pre, "total_score") {
                Err(v @ sea_orm::TryGetError::DbErr(_)) => {
                    return Err(v);
                }
                v => v,
            };
            Ok(Self {
                team: team?,
                total_score: total_score?,
            })
        }
    }
    pub struct NormalSpcDataAvg {
        pub team: i32,
        pub game_id: i32,
    }
    #[automatically_derived]
    impl sea_orm::FromQueryResult for NormalSpcDataAvg {
        fn from_query_result(
            row: &sea_orm::QueryResult,
            pre: &str,
        ) -> std::result::Result<Self, sea_orm::DbErr> {
            Ok(Self::from_query_result_nullable(row, pre)?)
        }
        fn from_query_result_nullable(
            row: &sea_orm::QueryResult,
            pre: &str,
        ) -> std::result::Result<Self, sea_orm::TryGetError> {
            let team = match row.try_get_nullable(pre, "team") {
                Err(v @ sea_orm::TryGetError::DbErr(_)) => {
                    return Err(v);
                }
                v => v,
            };
            let game_id = match row.try_get_nullable(pre, "game_id") {
                Err(v @ sea_orm::TryGetError::DbErr(_)) => {
                    return Err(v);
                }
                v => v,
            };
            Ok(Self {
                team: team?,
                game_id: game_id?,
            })
        }
    }
    async fn prim_average_game(
        model: Box<dyn YearOp>,
        event_code: &String,
        db: &DatabaseConnection,
    ) -> Result<Vec<GamesAvg>, DbErr> {
        let select_avg_score: Vec<NormalGenDataAvg> = genertic_header::Entity::find()
            .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()))
            .filter(genertic_header::Column::EventCode.eq(event_code))
            .select_only()
            .column_as(genertic_header::Column::TotalScore.avg(), "total_score")
            .column_as(genertic_header::Column::Team, "team")
            .group_by(genertic_header::Column::Team)
            .into_model::<NormalGenDataAvg>()
            .all(db)
            .await?;
        let ids: Vec<NormalSpcDataAvg> = genertic_header::Entity::find()
            .filter(genertic_header::Column::GameTypeId.eq(model.get_year_id()))
            .filter(genertic_header::Column::EventCode.eq(event_code))
            .select_only()
            .column(genertic_header::Column::GameId)
            .column(genertic_header::Column::Team)
            .into_model::<NormalSpcDataAvg>()
            .all(db)
            .await?;
        let data: Vec<(i32, Vec<i32>)> = ids
            .into_iter()
            .into_group_map_by(|record| record.team)
            .into_iter()
            .map(|(team, records)| {
                (team, records.into_iter().map(|r| r.game_id).collect())
            })
            .collect();
        let avg_map: HashMap<i32, f32> = select_avg_score
            .into_iter()
            .map(|x| (x.team, x.total_score))
            .collect();
        let a: Vec<(i32, GamesAvgSpecific)> = model.average_team(data, db).await?;
        let mut done: Vec<GamesAvg> = Vec::with_capacity(a.len());
        for spc in a {
            let avg = avg_map
                .get(&spc.0)
                .ok_or(DbErr::AttrNotSet("Could not find avg data".to_string()))?;
            done.push(GamesAvg {
                team: spc.0,
                total_score: *avg,
                game: spc.1,
            });
        }
        Ok(done)
    }
    async fn prim_get_game(
        model: Box<dyn YearOp>,
        id: i32,
        db: &DatabaseConnection,
    ) -> Result<GamesFull, DbErr> {
        let header = match genertic_header::Entity::find_by_id(id).one(db).await? {
            None => {
                return Err(DbErr::RecordNotFound("Could not find".to_string()));
            }
            Some(a) => a,
        };
        let game = model.get(header.game_id, db).await?;
        let right_header = to_full_match(header, db).await?;
        Ok(GamesFull {
            header: right_header,
            game,
        })
    }
    async fn prim_edit_game(
        model: Box<dyn YearOp>,
        edit: GamesEdit,
        db: &DatabaseConnection,
    ) -> Result<(), DbErr> {
        let game_id = to_full_am(edit.header, db).await?.update(db).await?.game_id;
        model.edit(game_id, edit.game, db).await?;
        Ok(())
    }
    pub struct GamesInserts {
        pub header: HeaderInsert,
        pub game: GamesInsertsSpecific,
    }
    pub struct HeaderGraph {
        pub time: DateTime<Local>,
        pub total_score: i32,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for HeaderGraph {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "HeaderGraph",
                    false as usize + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "time",
                    &self.time,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "total_score",
                    &self.total_score,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    pub struct GamesGraph {
        pub header: HeaderGraph,
        pub game: GamesGraphSpecific,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for GamesGraph {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "GamesGraph",
                    false as usize + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "header",
                    &self.header,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "game",
                    &self.game,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    pub struct GamesAvg {
        pub team: i32,
        pub total_score: f32,
        pub game: GamesAvgSpecific,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for GamesAvg {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "GamesAvg",
                    false as usize + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "team",
                    &self.team,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "total_score",
                    &self.total_score,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "game",
                    &self.game,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    pub struct GamesFull {
        pub header: HeaderFull,
        pub game: GamesFullSpecific,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for GamesFull {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "GamesFull",
                    false as usize + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "header",
                    &self.header,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "game",
                    &self.game,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    pub struct GamesEdit {
        header: HeaderFullEdit,
        game: GamesEditSpecific,
    }
    pub struct SearchParam {
        pub user: Option<String>,
        pub team: Option<i32>,
        pub is_ab_team: Option<bool>,
        pub match_id: Option<i32>,
        pub set: Option<i32>,
        pub total_score: Option<i32>,
        pub event_code: Option<String>,
        pub tournament_level: Option<TournamentLevels>,
        pub station: Option<Stations>,
        pub year: i32,
    }
    pub struct HeaderFull {
        pub id: i32,
        pub user: String,
        pub team: i32,
        pub is_ab_team: bool,
        pub match_id: i32,
        pub set: i32,
        pub total_score: i32,
        pub event_code: String,
        pub tournament_level: TournamentLevels,
        pub station: Stations,
        pub created_at: DateTime<Local>,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for HeaderFull {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "HeaderFull",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "id",
                    &self.id,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "user",
                    &self.user,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "team",
                    &self.team,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "is_ab_team",
                    &self.is_ab_team,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "match_id",
                    &self.match_id,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "set",
                    &self.set,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "total_score",
                    &self.total_score,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "event_code",
                    &self.event_code,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "tournament_level",
                    &self.tournament_level,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "station",
                    &self.station,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "created_at",
                    &self.created_at,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    pub struct HeaderFullEdit {
        pub id: Option<i32>,
        pub user: Option<String>,
        pub team: Option<i32>,
        pub is_ab_team: Option<bool>,
        pub match_id: Option<i32>,
        pub set: Option<i32>,
        pub total_score: Option<i32>,
        pub event_code: Option<String>,
        pub tournament_level: Option<TournamentLevels>,
        pub station: Option<Stations>,
        pub created_at: Option<DateTime<Local>>,
    }
    async fn to_full_am(
        header: HeaderFullEdit,
        db: &DatabaseConnection,
    ) -> Result<genertic_header::ActiveModel, DbErr> {
        let created_at;
        if let Some(c) = header.created_at {
            created_at = Some(c.naive_local());
        } else {
            created_at = None;
        }
        let username: Option<Uuid>;
        if let Some(name) = header.user {
            username = match auth::get_by_user::get_by_username(&name, db).await {
                Ok(a) => Some(a),
                Err(a) => {
                    match a {
                        AuthGetUuidError::UserIsNotHere => {
                            return Err(DbErr::Custom("User was not found".to_string()));
                        }
                        AuthGetUuidError::DatabaseError(db_err) => {
                            return Err(db_err);
                        }
                    }
                }
            };
        } else {
            username = None;
        }
        Ok(genertic_header::ActiveModel {
            id: header.id.map(Set).unwrap_or(NotSet),
            user: username.map(Set).unwrap_or(NotSet),
            team: header.team.map(Set).unwrap_or(NotSet),
            is_ab_team: header.is_ab_team.map(Set).unwrap_or(NotSet),
            match_id: header.match_id.map(Set).unwrap_or(NotSet),
            set: header.set.map(Set).unwrap_or(NotSet),
            total_score: header.total_score.map(Set).unwrap_or(NotSet),
            event_code: header.event_code.map(Set).unwrap_or(NotSet),
            tournament_level: header.tournament_level.map(Set).unwrap_or(NotSet),
            station: header.station.map(Set).unwrap_or(NotSet),
            created_at: created_at.map(Set).unwrap_or(NotSet),
            game_type_id: NotSet,
            game_id: NotSet,
        })
    }
    pub async fn insert_game(
        data: &GamesInserts,
        db: &DatabaseConnection,
    ) -> Result<i32, DbErr> {
        let game = game_dispatch(SETTINGS.year);
        prim_insert_game(data, game, db).await
    }
    pub async fn graph_game(
        team: &i32,
        event_code: &Option<String>,
        db: &DatabaseConnection,
    ) -> Result<Vec<GamesGraph>, DbErr> {
        let game = game_dispatch(SETTINGS.year);
        prim_graph_game(game, team, event_code, db).await
    }
    pub async fn search_game(
        param: &SearchParam,
        db: &DatabaseConnection,
    ) -> Result<Vec<GamesFull>, DbErr> {
        let game = game_dispatch(SETTINGS.year);
        prim_search_game(game, param, db).await
    }
    pub async fn average_game(
        event_code: &String,
        db: &DatabaseConnection,
    ) -> Result<Vec<GamesAvg>, DbErr> {
        let game = game_dispatch(SETTINGS.year);
        prim_average_game(game, event_code, db).await
    }
    pub async fn get_game(id: i32, db: &DatabaseConnection) -> Result<GamesFull, DbErr> {
        let game = game_dispatch(SETTINGS.year);
        prim_get_game(game, id, db).await
    }
    pub async fn delete_game(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
        let game = game_dispatch(SETTINGS.year);
        game.delete(id, db).await
    }
    pub async fn edit_game(
        edit: GamesEdit,
        db: &DatabaseConnection,
    ) -> Result<(), DbErr> {
        let game = game_dispatch(SETTINGS.year);
        prim_edit_game(game, edit, db).await
    }
    pub enum GamesFullSpecific {
        ExampleGame(crate::backenddb::example_game::Model),
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for GamesFullSpecific {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    GamesFullSpecific::ExampleGame(ref __field0) => {
                        _serde::Serializer::serialize_newtype_variant(
                            __serializer,
                            "GamesFullSpecific",
                            0u32,
                            "ExampleGame",
                            __field0,
                        )
                    }
                }
            }
        }
    };
    pub enum GamesAvgSpecific {
        ExampleGame(crate::backenddb::example_game::Avg),
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for GamesAvgSpecific {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    GamesAvgSpecific::ExampleGame(ref __field0) => {
                        _serde::Serializer::serialize_newtype_variant(
                            __serializer,
                            "GamesAvgSpecific",
                            0u32,
                            "ExampleGame",
                            __field0,
                        )
                    }
                }
            }
        }
    };
    pub enum GamesGraphSpecific {
        ExampleGame(crate::backenddb::example_game::Graph),
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for GamesGraphSpecific {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    GamesGraphSpecific::ExampleGame(ref __field0) => {
                        _serde::Serializer::serialize_newtype_variant(
                            __serializer,
                            "GamesGraphSpecific",
                            0u32,
                            "ExampleGame",
                            __field0,
                        )
                    }
                }
            }
        }
    };
    pub enum GamesInsertsSpecific {
        ExampleGame(crate::backenddb::example_game::Insert),
    }
    pub enum GamesEditSpecific {
        ExampleGame(crate::backenddb::example_game::Edit),
    }
    fn game_dispatch(year_id: i32) -> Box<dyn YearOp> {
        match year_id {
            crate::backenddb::example_game::YEAR => {
                Box::new(crate::backenddb::example_game::Functions) as Box<dyn YearOp>
            }
            _ => {
                ::core::panicking::panic_fmt(
                    format_args!("Unknown year_id: {0}", year_id),
                );
            }
        }
    }
}

//! Live DPDG refresh.
//!
//! Recomputes the four DPDG columns for every main-defender row of a single
//! event from the **latest** event averages (every finalized game in the
//! event), in one set-based pass. It runs on the write path — after
//! [`publish`](super::db_work::publish) has inserted the match's rows — so reads
//! only ever read stored values.
//!
//! This is the "latest average" semantics — the sole DPDG writer on the write
//! path. It revisits the whole event so every row reflects the current averages
//! rather than a value frozen at one match's finalize time. The scan is limited
//! to one event (`event_code` + `game_type_id`).
//!
//! Only rows whose value actually changes are written (a null-safe
//! `IS DISTINCT FROM` guard), so a new match normally touches just the defender
//! rows that target the six teams whose averages moved.
//!
//! The statement is built with `sea-query` (run through SeaORM), not a raw
//! string, so column references stay tied to [`genertic_header::Column`].

use sea_orm::sea_query::{Alias, CommonTableExpression, Condition, Expr, Func, JoinType, Query, WithClause};
use sea_orm::sea_query::ExprTrait;
use sea_orm::sea_query::extension::postgres::PgExpr;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};

use crate::entity::genertic_header::Column as Hc;

// Column reference helpers for the aliased tables in the statement.
fn h(c: Hc) -> Expr {
    Expr::col((Alias::new("h"), c))
}
fn o(c: Hc) -> Expr {
    Expr::col((Alias::new("o"), c))
}
fn eo(name: &str) -> Expr {
    Expr::col((Alias::new("eo"), Alias::new(name)))
}
fn opp(name: &str) -> Expr {
    Expr::col((Alias::new("opp"), Alias::new(name)))
}

/// Recompute DPDG for every finalized defender row of `event_code` for year
/// `year_id` using the latest event averages. Returns the number of rows written.
///
/// Mirrors:
/// ```sql
/// WITH ev_avg AS (
///     SELECT team, is_ab_team, AVG(total_score) AS avg
///     FROM genertic_header WHERE ... GROUP BY team, is_ab_team
/// ),
/// opp AS (
///     SELECT h.id, (h.defence_target -> 'Bot' IS NULL) AS is_alliance,
///            COUNT(o.id), AVG(eo.avg - o.total_score),
///            AVG(CASE WHEN eo.avg = 0 THEN 0
///                     ELSE (eo.avg - o.total_score) / eo.avg * 100 END)
///     FROM genertic_header h
///     LEFT JOIN genertic_header o ON <same match> AND <opposite alliance> AND <target>
///     LEFT JOIN ev_avg eo ON eo.team = o.team AND eo.is_ab_team = o.is_ab_team
///     WHERE h.<event> AND h.defence_main GROUP BY h.id, h.defence_target
/// )
/// UPDATE genertic_header SET ...
/// FROM opp
/// WHERE genertic_header.id = opp.hid
///   AND <null-safe IS DISTINCT FROM across the four columns>
/// ```
pub async fn refresh_event_dpdg(
    event_code: &str,
    year_id: i32,
    db: &DatabaseConnection,
) -> Result<u64, DbErr> {
    let update = build_refresh(event_code, year_id);
    let res = db.execute(&update).await?;
    Ok(res.rows_affected())
}

/// Build the single-statement refresh as a sea-query `UPDATE ... FROM` with the
/// `ev_avg` / `opp` CTEs. Split out so the SQL shape can be unit-tested without
/// a database.
fn build_refresh(event_code: &str, year_id: i32) -> sea_orm::sea_query::UpdateStatement {
    // ── ev_avg: latest per-team event average ────────────────────────────────
    let ev_select = Query::select()
        .column(Hc::Team)
        .column(Hc::IsAbTeam)
        .expr_as(
            Func::cast_as(Func::avg(Expr::col(Hc::TotalScore)), Alias::new("float8")),
            Alias::new("avg"),
        )
        .from(Alias::new("genertic_header"))
        .and_where(Expr::col(Hc::GameTypeId).eq(year_id))
        .and_where(Expr::col(Hc::EventCode).eq(event_code))
        .and_where(Expr::col(Hc::IsPrescout).eq(false))
        .group_by_col(Hc::Team)
        .group_by_col(Hc::IsAbTeam)
        .take();

    let mut cte_ev = CommonTableExpression::new();
    cte_ev.table_name(Alias::new("ev_avg")).query(ev_select);

    // ── opp: one row per main defender with n / raw / pct ────────────────────
    // `-> 'Bot'` is NULL for an Alliance target (the stored value is the JSON
    // string "Alliance"), so this doubles as the is_alliance flag.
    let bot = h(Hc::DefenceTarget).get_json_field("Bot");
    let is_alliance = bot.clone().is_null();

    let target_match = Condition::any()
        // Alliance target: every opponent is in play.
        .add(is_alliance.clone())
        // Bot target: only the opposing row whose team + ab-flag match.
        .add(
            Condition::all()
                .add(
                    bot.clone()
                        .cast_json_field("number")
                        .cast_as(Alias::new("int"))
                        .eq(o(Hc::Team)),
                )
                .add(
                    bot.clone()
                        .cast_json_field("is_ab_team")
                        .cast_as(Alias::new("bool"))
                        .eq(o(Hc::IsAbTeam)),
                ),
        );

    // `station` is a Postgres enum (`stations`); a prepared statement would bind
    // the string literals below as `text`, and PostgreSQL has no
    // `stations = text` operator (SQLSTATE 42883). Cast the column to text.
    let h_is_red = h(Hc::Station)
        .cast_as(Alias::new("text"))
        .is_in(["red1", "red2", "red3"]);
    let o_is_red = o(Hc::Station)
        .cast_as(Alias::new("text"))
        .is_in(["red1", "red2", "red3"]);

    let match_join = Condition::all()
        .add(o(Hc::GameTypeId).eq(h(Hc::GameTypeId)))
        .add(o(Hc::EventCode).eq(h(Hc::EventCode)))
        .add(o(Hc::MatchId).eq(h(Hc::MatchId)))
        .add(o(Hc::Set).eq(h(Hc::Set)))
        .add(o(Hc::TournamentLevel).eq(h(Hc::TournamentLevel)))
        .add(h_is_red.ne(o_is_red))
        .add(target_match);

    let eo_join = Condition::all()
        .add(eo("team").eq(o(Hc::Team)))
        .add(eo("is_ab_team").eq(o(Hc::IsAbTeam)));

    let eo_avg = eo("avg");
    let o_total = o(Hc::TotalScore);
    let pct_case = Expr::case(eo_avg.clone().eq(0), Expr::val(0)).finally(
        eo_avg
            .clone()
            .sub(o_total.clone())
            .div(eo_avg.clone())
            .mul(100),
    );

    let opp_select = Query::select()
        .expr_as(h(Hc::Id), Alias::new("hid"))
        .expr_as(is_alliance.clone(), Alias::new("is_alliance"))
        .expr_as(Func::count(o(Hc::Id)), Alias::new("n"))
        .expr_as(Func::avg(eo_avg.clone().sub(o_total.clone())), Alias::new("raw"))
        .expr_as(Func::avg(pct_case), Alias::new("pct"))
        .from_as(Alias::new("genertic_header"), Alias::new("h"))
        .join_as(
            JoinType::LeftJoin,
            Alias::new("genertic_header"),
            Alias::new("o"),
            match_join,
        )
        .join_as(JoinType::LeftJoin, Alias::new("ev_avg"), Alias::new("eo"), eo_join)
        .and_where(h(Hc::GameTypeId).eq(year_id))
        .and_where(h(Hc::EventCode).eq(event_code))
        .and_where(h(Hc::IsPrescout).eq(false))
        .and_where(h(Hc::DefenceMain).eq(true))
        .group_by_col((Alias::new("h"), Hc::Id))
        .group_by_col((Alias::new("h"), Hc::DefenceTarget))
        .take();

    let mut cte_opp = CommonTableExpression::new();
    cte_opp.table_name(Alias::new("opp")).query(opp_select);

    // ── UPDATE ... FROM opp ──────────────────────────────────────────────────
    let no_opponents = opp("n").eq(0);
    let to_team = Condition::any()
        .add(no_opponents.clone())
        .add(opp("is_alliance").eq(true));
    let to_alliance = Condition::any()
        .add(no_opponents.clone())
        .add(opp("is_alliance").eq(false));

    // Targets are `real` columns, so cast the computed value to `real` before
    // both the SET and the change-guard. Otherwise the guard compares the stored
    // float4 against an unrounded float8 and every row looks changed.
    let team_pct: Expr = Expr::case(to_team.clone(), Expr::null())
        .finally(opp("pct"))
        .cast_as(Alias::new("real"));
    let team_raw: Expr = Expr::case(to_team.clone(), Expr::null())
        .finally(opp("raw"))
        .cast_as(Alias::new("real"));
    let alliance_pct: Expr = Expr::case(to_alliance.clone(), Expr::null())
        .finally(opp("pct"))
        .cast_as(Alias::new("real"));
    let alliance_raw: Expr = Expr::case(to_alliance.clone(), Expr::null())
        .finally(opp("raw"))
        .cast_as(Alias::new("real"));

    // Only write rows whose stored value actually changes. `IS DISTINCT FROM` is
    // null-safe (a NULL <-> value transition counts as distinct); it is expanded
    // here because sea-query has no dedicated operator for it.
    let changed = Condition::any()
        .add(is_distinct(Expr::col(Hc::DpdgTeam), team_pct.clone()))
        .add(is_distinct(Expr::col(Hc::DpdgTeamRaw), team_raw.clone()))
        .add(is_distinct(Expr::col(Hc::DpdgAlliance), alliance_pct.clone()))
        .add(is_distinct(Expr::col(Hc::DpdgAllianceRaw), alliance_raw.clone()));

    let mut clause = WithClause::new();
    clause.cte(cte_ev).cte(cte_opp);

    let mut update = Query::update();
    update
        .table(Alias::new("genertic_header"))
        .value(Hc::DpdgTeam, team_pct)
        .value(Hc::DpdgTeamRaw, team_raw)
        .value(Hc::DpdgAlliance, alliance_pct)
        .value(Hc::DpdgAllianceRaw, alliance_raw)
        .from(Alias::new("opp"))
        .and_where(Expr::col((Alias::new("genertic_header"), Hc::Id)).eq(opp("hid")))
        .cond_where(changed)
        .with_cte(clause);

    update
}

/// `old IS DISTINCT FROM new`, expanded to null-safe builder expressions.
fn is_distinct(old: Expr, new: Expr) -> Condition {
    Condition::any()
        .add(old.clone().ne(new.clone()))
        .add(
            Condition::all()
                .add(old.clone().is_null())
                .add(new.clone().is_not_null()),
        )
        .add(
            Condition::all()
                .add(old.is_not_null())
                .add(new.is_null()),
        )
}

#[cfg(test)]
mod tests {
    use super::build_refresh;
    use sea_orm::sea_query::PostgresQueryBuilder;

    #[test]
    fn refresh_sql_shape() {
        let sql = build_refresh("2026caasv", 2026).to_string(PostgresQueryBuilder);
        println!("{sql}");

        assert!(sql.contains("WITH \"ev_avg\" AS") || sql.contains("WITH ev_avg AS"));
        assert!(sql.contains("UPDATE \"genertic_header\""));
        assert!(sql.contains("FROM \"opp\""));
        assert!(sql.contains("LEFT JOIN \"genertic_header\" AS \"o\""));
        assert!(sql.contains("LEFT JOIN \"ev_avg\" AS \"eo\""));
        assert!(sql.contains("-> 'Bot'"));
        assert!(sql.contains("CASE WHEN"));
        // Null-safe change guard is present.
        assert!(sql.contains("\"dpdg_team\" IS NULL") && sql.contains("\"dpdg_team\" IS NOT NULL"));
    }
}

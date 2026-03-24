use std::collections::HashMap;

use rocket::State;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::entity::{game_scouts, mvp_scouters, upcoming_game, users};

// For backend testing only — not mounted at /api/
// Usage: GET /check_status/<upcoming_game_id>
#[get("/check_status/<id>")]
pub async fn check_status(id: i32, db: &State<DatabaseConnection>) -> String {
    let game = match upcoming_game::Entity::find_by_id(id).one(db.inner()).await {
        Ok(Some(g)) => g,
        Ok(None) => return format!("No game found with id={id}"),
        Err(e) => return format!("DB error: {e}"),
    };

    let all_scouts = match game_scouts::Entity::find()
        .filter(game_scouts::Column::GameId.eq(id))
        .all(db.inner())
        .await
    {
        Ok(s) => s,
        Err(e) => return format!("DB error fetching scouts: {e}"),
    };

    // Batch-fetch all usernames referenced by this game's scouts
    let scouter_ids: Vec<Uuid> = all_scouts.iter().map(|s| s.scouter_id).collect();
    let user_map: HashMap<Uuid, String> = match users::Entity::find()
        .filter(users::Column::Id.is_in(scouter_ids))
        .all(db.inner())
        .await
    {
        Ok(us) => us.into_iter().map(|u| (u.id, u.name)).collect(),
        Err(_) => HashMap::new(),
    };

    let name_of = |uuid: Uuid| -> String {
        user_map.get(&uuid).cloned().unwrap_or_else(|| uuid.to_string())
    };

    let pending_scouts: Vec<&game_scouts::Model> =
        all_scouts.iter().filter(|s| s.game_midway.is_none()).collect();
    let done_scouts: Vec<&game_scouts::Model> =
        all_scouts.iter().filter(|s| s.game_midway.is_some()).collect();

    // MVPs
    let mvp_red_done = match game.mvp_id_red {
        None => false,
        Some(mvp_id) => match mvp_scouters::Entity::find_by_id(mvp_id).one(db.inner()).await {
            Ok(Some(m)) => m.data.is_some(),
            _ => false,
        },
    };
    let mvp_blue_done = match game.mvp_id_blue {
        None => false,
        Some(mvp_id) => match mvp_scouters::Entity::find_by_id(mvp_id).one(db.inner()).await {
            Ok(Some(m)) => m.data.is_some(),
            _ => false,
        },
    };

    let total = all_scouts.len();
    let pending = pending_scouts.len();
    let done = done_scouts.len();

    let mut out = format!(
        "=== check_status for game id={id} (match={} set={}) ===\n\n",
        game.match_id, game.set
    );

    out += &format!("Scouts: {done}/{total} done, {pending} remaining\n");

    if !pending_scouts.is_empty() {
        out += "\nPending scouts:\n";
        for s in &pending_scouts {
            out += &format!(
                "  station={:?}  user={}  is_redo={}\n",
                s.station,
                name_of(s.scouter_id),
                s.is_redo
            );
        }
    }

    if !done_scouts.is_empty() {
        out += "\nDone scouts:\n";
        for s in &done_scouts {
            out += &format!(
                "  station={:?}  user={}\n",
                s.station,
                name_of(s.scouter_id)
            );
        }
    }

    out += &format!(
        "\nMVP red:  {}\n",
        if game.mvp_id_red.is_none() { "not assigned" } else if mvp_red_done { "done" } else { "pending" }
    );
    out += &format!(
        "MVP blue: {}\n",
        if game.mvp_id_blue.is_none() { "not assigned" } else if mvp_blue_done { "done" } else { "pending" }
    );

    let scouts_needed = pending;
    let mvps_needed = (!mvp_red_done as usize) + (!mvp_blue_done as usize);
    let total_needed = scouts_needed + mvps_needed;

    out += &format!(
        "\n→ {total_needed} submission(s) needed before check can run ({scouts_needed} scout(s), {mvps_needed} MVP(s))\n"
    );

    out
}

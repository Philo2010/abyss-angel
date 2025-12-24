
use rocket::State;
use rocket::{post, serde::json::Json};
use rocket_dyn_templates::{Template, context};
use sea_orm::{DatabaseConnection, EntityTrait, ModelTrait, PaginatorTrait};
use serde_json::Value;

use crate::SETTINGS;
use crate::upcoming_handler::{upcoming_game, upcoming_team};
use crate::user::YEARSINSERT;


#[post("/scout_form", data = "<body>")]
pub async fn scout_take(body: Json<Value>, db: &State<DatabaseConnection>) -> Template {
    let insrfunc = YEARSINSERT[&SETTINGS.year];

    let body_value = body.into_inner();

    match insrfunc(db.inner(), &body_value).await {
        Ok(_) => {
            Template::render("suc", context! {message: "Properly scouted"})
        },
        Err(a) => {
            let errormesage = format!("Error inserting!: {a}");
            return Template::render("error", context!{error: errormesage});
        }
    };

    if let Some(teamid) = body_value.get("teamid") {
        if let Some(teamidvalue) = teamid.as_i64() {
            let ent = match upcoming_team::Entity::find_by_id(teamidvalue as i32).one(db.inner()).await {
                Ok(Some(a)) => a,
                Ok(None) => {
                    return Template::render("error", context! {error: "Queued and removed, but unable to preform match check, expect funny beavhar"});
                },
                Err(a) => {
                    return Template::render("error", context! {error: format!("Queued and removed, but unable to preform match check, expect funny beavhar: {a}")});
                },
            };
            
            let res = upcoming_team::Entity::delete_by_id(teamidvalue as i32).exec(db.inner()).await;
            match res {
                Ok(_) =>  {
                    //Now we must check if that was the last one we truely had
                    let game = match upcoming_game::Entity::find_by_id(ent.game_id).one(db.inner()).await {
                        Ok(Some(a)) => a,
                        Ok(None) => {
                            return Template::render("error", context! {error: "Scouted, but failed to find a related game"});
                        },
                        Err(a) => {
                            return Template::render("error", context! {error: format!("Scouted, but failed to find a related game: {a}")});
                        },
                    };

                    let count = match game.find_related(upcoming_team::Entity).count(db.inner()).await {
                        Ok(a) => a,
                        Err(_) => {
                            return Template::render("error", context! {error: "Failed to get count!"});
                        },
                    };

                    if count == 0 {
                        match upcoming_game::Entity::delete_by_id(ent.game_id).exec(db.inner()).await {
                            Ok(_) => {
                                return Template::render("suc", context! {message: "Properly scouted"});
                            },
                            Err(_) => {
                                return Template::render("error", context! {error: "Able to insert into database, but unable to remove queued match!"});
                            },
                        }
                    } else {
                        return Template::render("suc", context! {message: "Properly scouted"});
                    }
                }, 
                Err(a) => {
                    let errormesage = format!("Error deleting upcoming entry!: {a}");
                    return Template::render("error", context!{error: errormesage});
                },
            }
        } else {
            return Template::render("error", context! {error: "Able to insert into database, but unable to remove queued entry!"});
        }
    } else {
        //Dont do anything, most likey a manual scout
        return Template::render("suc", context! {message: "Properly scouted, however, please avoid using manual scout (unless directly requested)!"});
    }
}

use rocket::{post, serde::json::Json};
use serde_json::Value;

use crate::SETTINGS;
use crate::user::YEARSINSERT;


#[post("/scout_form", data = "<body>")]
async fn scout_take(body: Json<Value>) -> String {
    let insrfunc = YEARSINSERT[&SETTINGS.year];

    "Goon".to_string() //Get off my ass
}

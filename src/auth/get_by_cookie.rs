use std::str::FromStr;

use rocket::http::CookieJar;
use uuid::Uuid;




pub fn get(cookies: &CookieJar<'_>) -> Option<Uuid> { 
    let val = match cookies.get(crate::auth::UUID_COOKIE_NAME).map(|c| c.value()) {
        Some(a) => a,
        None => {
            return None;
        },
    };

    let uuid = match Uuid::from_str(val) {
        Ok(a) => a,
        Err(_) => {
            return None
        },
    };

    Some(uuid)
}
//NOTE: Scouters don't have there own accounts, they only use the select scout system (see src/upcoming_handler)

const UUID_COOKIE_NAME: &str = "uuiduser";

pub mod admin;
pub mod create_user;
pub mod login;
pub mod check;
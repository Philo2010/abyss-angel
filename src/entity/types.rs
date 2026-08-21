use schemars::JsonSchema;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq, Clone, Copy, JsonSchema, Serialize, Deserialize, Debug)]
pub struct Team {
    pub number: i32,
    pub is_ab_team: bool,
}

/// What a main defender spent the match defending.
///
/// Only meaningful when the game is flagged as the main defender — it is `Some`
/// exactly when that flag is set, and `None` otherwise. `Alliance` means the
/// defence was spread across the whole opposing alliance; `Bot` means a single
/// opposing robot was targeted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, FromJsonQueryResult)]
pub enum DefenceTarget {
    Alliance,
    Bot(Team),
}

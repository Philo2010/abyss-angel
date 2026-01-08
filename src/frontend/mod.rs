use serde::Serialize;
use rocket_okapi::JsonSchema;

pub mod graph;
pub mod averages;
pub mod search;
pub mod delete;
pub mod pit;
pub mod snowgrave;
pub mod scoutwarn;

#[derive(Serialize, JsonSchema)]
#[serde(tag = "status", content = "message")]
pub enum ApiResult<T: Serialize + JsonSchema> {
    Success(T),
    Error(String),
}
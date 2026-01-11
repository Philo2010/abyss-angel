//This is the main file where snow grave will be writen
//Snowgrave is an impl of the MVP scouter system derived by Max to Abyss Angel
//This will replace any direct insertsion into the database!!!!
//! I REPEAT, FROM NOW ON, ALL INSERTSION INVOLING SCOUTING (not pit) MUST BE HANDLED BY SNOWGRAVE


pub mod snowgrave_que;
pub mod snowgrave_insert_scouters;
pub mod get_games_from_scouter;
pub mod datatypes;
pub mod insert_mvp_data;
pub mod check_complete;
pub mod check;
pub mod hydrate;
pub mod cast_snowgrave;
pub mod insert_scout_data;
pub mod blue;
pub mod snowgrave_que_only_playoff;
pub mod snowgrave_edit_scouter;
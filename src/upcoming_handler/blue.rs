//This file is used in a few ways, mainly with 

use reqwest::Client;
use serde::Deserialize;
use reqwest::header::HeaderMap;

use crate::SETTINGS;

#[derive(Debug, Deserialize)]
pub struct TbaMatch {
    pub comp_level: String,
    pub match_number: i32,
    pub set_number: i32,
    pub alliances: Alliances,
}

#[derive(Debug, Deserialize)]
pub struct Alliances {
    pub red: Alliance,
    pub blue: Alliance,
}

#[derive(Debug, Deserialize)]
pub struct Alliance {
    pub score: Option<i32>,
    pub team_keys: Vec<String>,
}

pub async fn pull_from_blue(client: &Client, event_code: &String) -> Result<Vec<TbaMatch>, reqwest::Error> {
    //https://www.thebluealliance.com/api/v3/event/2025tacy/matches/simple

    let mut auth_headers =  reqwest::header::HeaderMap::new();
    auth_headers.insert("accept", "application/json".parse().unwrap());
    auth_headers.insert("X-TBA-Auth-Key", SETTINGS.blue_api_key.to_string().parse().unwrap());

    //Make a request to get the major data
    let request = client.get(format!("https://www.thebluealliance.com/api/v3/event/{}/matches/simple", event_code))
        .headers(auth_headers.clone()).send().await?;

    let body: Vec<TbaMatch> = request.json().await?;
    

    Ok(body)
}
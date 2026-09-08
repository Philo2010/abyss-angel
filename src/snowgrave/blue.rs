//This file is used in a few ways, mainly with 

use reqwest::Client;
use serde::Deserialize;

use crate::{SETTINGS, entity::types::Team};

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
pub struct EventRanking {
    pub rankings: Vec<EventTeam>,
}


#[derive(Debug)]
pub struct EventTeamNice {
    pub team_key: Team,
    pub rank: i32,
}

pub struct EventRankingNice {
    pub rankings: Vec<EventTeamNice>,
}


#[derive(Debug, Deserialize)]
pub struct EventTeam {
    pub team_key: String,
    pub rank: i32,
}

pub async fn get_ranking_from_blue(client: &Client, event_code: &String) ->Result<EventRankingNice, reqwest::Error> {
    
    let mut auth_headers =  reqwest::header::HeaderMap::new();
    auth_headers.insert("accept", "application/json".parse().unwrap());
    auth_headers.insert("X-TBA-Auth-Key", SETTINGS.blue_api_key.to_string().parse().unwrap());



    let request = client.get(format!("https://www.thebluealliance.com/api/v3/event/{}/rankings", event_code))
        .headers(auth_headers.clone()).send().await?;

    let data: EventRanking = request.json().await?;

    let ranking_proper: Vec<EventTeamNice> = data.rankings.into_iter().map(|x| {
        let is_b_team: bool;
        let team_string: String;
        if x.team_key.ends_with('B') {
            is_b_team = true;
            team_string = x.team_key[3..x.team_key.len() - 1].to_string();
        } else {
            is_b_team = false;
            team_string = x.team_key[3..].to_string();
        };
        let team_number: i32 = team_string.parse().unwrap();

        EventTeamNice {
            team_key: Team {
                number: team_number,
                is_ab_team: is_b_team,
            },
            rank: x.rank,
        }
    }).collect();

    Ok(EventRankingNice { rankings: ranking_proper })
}


#[derive(Debug, Deserialize)]
pub struct Alliance {
    #[allow(dead_code)] //Im like 80% sure im gonna need to use the score *later* so \(:/
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
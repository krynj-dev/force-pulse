// riot.rs
use reqwest::blocking::Client;
use serde_json::Value;

pub fn fetch_match(api_key: &str, region: &str, match_id: &str) -> Result<Value, reqwest::Error> {
    let url = format!("https://{region}.api.riotgames.com/lol/match/v5/matches/{match_id}");

    let client = Client::new();
    let res = client
        .get(url)
        .header("X-Riot-Token", api_key)
        .send()?
        .error_for_status()?; // fails on non-200

    res.json()
}

pub fn fetch_account(
    api_key: &str,
    region: &str,
    game_name: &str,
    tag_line: &str,
) -> Result<Value, reqwest::Error> {
    let url = format!(
        "https://{region}.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{game_name}/{tag_line}"
    );

    let client = Client::new();
    let res = client
        .get(url)
        .header("X-Riot-Token", api_key)
        .send()?
        .error_for_status()?; // fails on non-200

    res.json()
}

pub fn fetch_match_ids(api_key: &str, region: &str, puuid: &str) -> Result<Value, reqwest::Error> {
    let url =
        format!("https://{region}.api.riotgames.com/lol/match/v5/matches/by-puuid/{puuid}/ids");

    let client = Client::new();
    let res = client
        .get(url)
        .query(&[("type", "tourney"), ("count", "14")])
        .header("X-Riot-Token", api_key)
        .send()?
        .error_for_status()?; // fails on non-200

    res.json()
}

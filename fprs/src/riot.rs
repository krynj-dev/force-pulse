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

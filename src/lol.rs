use serde_json;
use std::env;

use serde_json::Value;

pub async fn get_summoner_id(summoner_name: String) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://br1.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}",
        summoner_name.replace(" ", "%20"),
        env::var("RIOT_API_KEY").unwrap()
    );

    let res = reqwest::get(url).await?;

    let summoner = res.text().await?;

    let summoner: Value = serde_json::from_str(&summoner).unwrap();

    Ok(summoner["id"].as_str().unwrap().to_string())
}

pub async fn get_summoner_rank(summoner_id: String) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://br1.api.riotgames.com/lol/league/v4/entries/by-summoner/{}?api_key={}",
        summoner_id,
        env::var("RIOT_API_KEY").unwrap()
    );
    let res = reqwest::get(url).await?;
    let summoner = res.text().await?;

    let summoner: Value = serde_json::from_str(&summoner).unwrap();

    if summoner[0]["queueType"].as_str().unwrap() == "RANKED_SOLO_5x5" {
        Ok(summoner[0]["tier"].as_str().unwrap().to_string())
    } else if summoner[1]["queueType"].as_str().unwrap() == "RANKED_SOLO_5x5" {
        Ok(summoner[1]["tier"].as_str().unwrap().to_string())
    } else {
        Ok("Unranked".to_string())
    }
}

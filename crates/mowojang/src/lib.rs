use database::player::PlayerInfo;
use uuid::Uuid;

pub async fn check_uuid(uuid: Uuid) -> Option<PlayerInfo> {
    match reqwest::get(format!(
        "https://mowojang.matdoes.dev/{}",
        urlencoding::encode(&uuid.to_string())
    ))
    .await
    {
        Ok(response) => match response.json().await {
            Ok(json) => json,
            Err(_) => None,
        },
        Err(_) => None,
    }
}

pub async fn check_username(username: &str) -> Option<PlayerInfo> {
    match reqwest::get(format!(
        "https://mowojang.matdoes.dev/{}",
        urlencoding::encode(username)
    ))
    .await
    {
        Ok(response) => match response.json().await {
            Ok(json) => json,
            Err(_) => None,
        },
        Err(_) => None,
    }
}

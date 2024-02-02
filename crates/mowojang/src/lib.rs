use database::player::PlayerInfo;
use uuid::Uuid;

pub fn valid_java_username(username: &str) -> bool {
    (3..=16).contains(&username.len())
        && username
            .chars()
            .all(|char| (char.is_ascii_alphanumeric() || char == '_'))
}

pub fn valid_geyser_username(username: &str) -> bool {
    let mut chars = username.chars();
    (3..=12).contains(&username.len().saturating_sub(1))
        && chars.next() == Some('.')
        && chars.next().expect("wha").is_ascii_alphabetic()
        && chars.all(|char| (char.is_ascii_alphanumeric() || char == '_'))
}

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

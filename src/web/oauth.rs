use super::ServerState;
use crate::database::{discord_user::DiscordUserInfo, DatabaseConnection};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    RedirectUrl, Scope, StandardTokenResponse, TokenResponse, TokenUrl,
};
use reqwest::header::SET_COOKIE;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::PgPool;
use tokio::sync::MutexGuard;

#[derive(Debug, Deserialize, Clone)]
pub struct Oauth2Parameters {
    pub code: String,
}

pub async fn oauth2(
    server_state: State<ServerState>,
    oauth2_parameters: Query<Oauth2Parameters>,
) -> impl IntoResponse {
    match try_oauth(oauth2_parameters.0.clone()).await {
        Ok((_result, _discord_user_info, _discord_guild_member)) => {
            let db: MutexGuard<DatabaseConnection> = server_state.0.db.lock().await;
            let _pool: &PgPool = &db.pool;

            let mut headers = HeaderMap::new();
            headers.insert(SET_COOKIE, "a=b".parse().unwrap());
            (headers, Redirect::to("/discord_login.html"))
        }
        Err(_) => (HeaderMap::new(), Redirect::to("/discord_login.html")),
    }
}

pub const BASE_AUTHORIZE_URI: &str = "https://discord.com/api/oauth2/authorize";
// pub const BASE_REVOKE_URI: &str = "https://discord.com/api/oauth2/token/revoke";
pub const BASE_TOKEN_URI: &str = "https://discord.com/api/oauth2/token";

async fn try_oauth(
    oauth2_parameters: Oauth2Parameters,
) -> eyre::Result<(
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    DiscordUserInfo,
    DiscordGuildMember,
)> {
    let redirect_uri = std::env::var("REDIRECT_URI").unwrap();
    let client_id = std::env::var("CLIENT_ID").unwrap();
    let client_secret = std::env::var("CLIENT_SECRET").unwrap();
    let guild_id = std::env::var("GUILD_ID").unwrap();

    let client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
        AuthUrl::new(BASE_AUTHORIZE_URI.to_string())?,
        Some(TokenUrl::new(BASE_TOKEN_URI.to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?);

    let (_auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("guilds".to_string()))
        .url();

    let token_result = client
        .exchange_code(AuthorizationCode::new(oauth2_parameters.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await?;

    let client = reqwest::Client::new();
    let discord_user_info = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await?
        .json::<DiscordUserInfo>()
        .await?;
    let discord_guild_member_info = client
        .get(format!(
            "https://discord.com/api/users/@me/guilds/{guild_id}/member"
        ))
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await?
        .json::<DiscordGuildMember>()
        .await?;

    Ok((token_result, discord_user_info, discord_guild_member_info))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum DiscordGuildMember {
    Member(Box<DiscordGuildMemberInfo>),
    Error(DiscordUnknownGuildInfo),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DiscordUnknownGuildInfo {
    pub message: String,
    pub code: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DiscordGuildMemberInfo {
    pub avatar: Option<String>,
    #[serde(deserialize_with = "deserialize_option")]
    pub communication_disabled_until: Option<time::OffsetDateTime>,
    pub flags: i32,
    pub joined_at: String,
    pub nick: Option<String>,
    pub pending: Option<bool>,
    #[serde(deserialize_with = "deserialize_option")]
    pub premium_since: Option<time::OffsetDateTime>,
    pub roles: Vec<String>,
    #[serde(deserialize_with = "deserialize_option")]
    pub unusual_dm_activity_until: Option<time::OffsetDateTime>,
    pub user: Option<DiscordUserInfo>,
    pub mute: bool,
    pub deaf: bool,
    pub bio: String,
    pub banner: Option<String>,
    pub permissions: Option<String>,
}

pub fn deserialize_option<'de, D>(deserializer: D) -> Result<Option<time::OffsetDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<time::OffsetDateTime>::deserialize(deserializer)
}

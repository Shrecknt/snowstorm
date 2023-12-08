use crate::{database::DatabaseConnection, modes::ScanningMode, ScannerState};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, Query, State, WebSocketUpgrade,
    },
    headers,
    response::IntoResponse,
    routing::{get, post},
    Form, Router, TypedHeader,
};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    RedirectUrl, Scope, StandardTokenResponse, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use simd_json::owned::Value;
use sqlx::PgPool;
use std::{collections::LinkedList, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};
use tokio::sync::{Mutex, MutexGuard};
use tower_http::services::{ServeDir, ServeFile};
#[cfg(debug_assertions)]
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
#[cfg(debug_assertions)]
use tracing_subscriber::prelude::*;

#[derive(Clone)]
pub struct ServerState {
    pub db: Arc<Mutex<DatabaseConnection>>,
    pub state: Arc<Mutex<ScannerState>>,
    pub task_queue: Arc<Mutex<LinkedList<(ScanningMode, Duration)>>>,
}

pub async fn start_server(
    db: Arc<Mutex<DatabaseConnection>>,
    state: Arc<Mutex<ScannerState>>,
    task_queue: Arc<Mutex<LinkedList<(ScanningMode, Duration)>>>,
) -> eyre::Result<()> {
    #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "snowstorm=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let server_state = ServerState {
        db,
        state,
        task_queue,
    };

    let serve_dir = ServeDir::new("web").not_found_service(ServeFile::new("web/404.html"));

    let routes = Router::new()
        .route("/ws", get(ws_handler))
        .route("/auth", post(authentication))
        .route("/oauth2", get(oauth2))
        .fallback_service(serve_dir.clone());
    let routes = routes.layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );
    let routes = routes.with_state(server_state);

    let listener = SocketAddr::from_str(&std::env::var("WEB_LISTEN_URL")?)?;
    axum::Server::bind(&listener)
        .serve(routes.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

#[allow(clippy::unused_async)]
async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    let _ = socket.send(Message::Ping(vec![1, 2, 3])).await;

    while let Some(message) = socket.recv().await {
        match message {
            Ok(message) => {
                let text = match &message {
                    Message::Text(text) => text.trim(),
                    Message::Close(_) => break,
                    _ => {
                        println!("non-text message: {:?}", message);
                        continue;
                    }
                };
                if text.is_empty() {
                    continue;
                }
                println!("len: '{}', bytes: '{:?}'", text.len(), text.as_bytes());
                println!("message: '{}'", text);

                let json: Value = match simd_json::deserialize(&mut text.as_bytes().to_owned()) {
                    Ok(json) => json,
                    Err(err) => {
                        let _ = socket
                            .send(Message::Text(format!(
                                "{{\"err\":\"{}\"}}",
                                err.to_string().replace('"', "\\\"")
                            )))
                            .await;
                        continue;
                    }
                };

                println!("json: {}", json);
            }
            Err(err) => {
                println!("err: {}", err);
                break;
            }
        }
    }

    println!("Websocket context {who} destroyed");
}

#[derive(Deserialize, Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

async fn authentication(
    server_state: State<ServerState>,
    credentials: Form<LoginInput>,
) -> impl IntoResponse {
    let db: MutexGuard<DatabaseConnection> = server_state.0.db.lock().await;
    let _pool: &PgPool = &db.pool;
    println!("credentials: {:?}", credentials.0);
}

#[derive(Debug, Deserialize, Clone)]
pub struct Oauth2Parameters {
    pub code: String,
}

async fn oauth2(
    server_state: State<ServerState>,
    oauth2_parameters: Query<Oauth2Parameters>,
) -> impl IntoResponse {
    let db: MutexGuard<DatabaseConnection> = server_state.0.db.lock().await;
    let _pool: &PgPool = &db.pool;
    println!("credentials: {:?}", oauth2_parameters.0);

    match try_oauth(oauth2_parameters.0.clone()).await {
        Ok((result, discord_user_info, discord_guild_member)) => {
            format!("{result:?}\n\n{discord_user_info:?}\n\n{discord_guild_member:?}")
        }
        Err(err) => err.to_string(),
    }
}

pub const BASE_AUTHORIZE_URI: &str = "https://discord.com/api/oauth2/authorize";
pub const BASE_REVOKE_URI: &str = "https://discord.com/api/oauth2/revoke";
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

#[derive(Deserialize, Debug)]
pub struct DiscordUserInfo {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub public_flags: Option<i32>,
    pub premium_type: Option<i32>,
    pub flags: Option<i32>,
    pub banner: Option<String>,
    pub accent_color: Option<i32>,
    pub global_name: Option<String>,
    pub avatar_decoration: Option<String>,
    pub banner_color: Option<String>,
    pub mfa_enabled: Option<bool>,
    pub locale: Option<String>,
    pub bot: Option<bool>,
    pub system: Option<bool>,
    pub verified: Option<bool>,
    pub email: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DiscordGuildMember {
    Member(DiscordGuildMemberInfo),
    Error(DiscordUnknownGuildInfo),
}

#[derive(Deserialize, Debug)]
pub struct DiscordUnknownGuildInfo {
    pub message: String,
    pub code: i32,
}

#[derive(Deserialize, Debug)]
pub struct DiscordGuildMemberInfo {
    pub avatar: Option<String>,
    pub communication_disabled_until: Option<i32>,
    pub flags: i32,
    pub joined_at: String,
    pub nick: Option<String>,
    pub pending: Option<bool>,
    pub premium_since: Option<String>,
    pub roles: Vec<String>,
    pub unusual_dm_activity_until: Option<String>,
    pub user: Option<DiscordUserInfo>,
    pub mute: bool,
    pub deaf: bool,
    pub bio: String,
    pub banner: Option<String>,
    pub permissions: Option<String>,
}

use crate::{database::DatabaseConnection, modes::ScanningMode, ScannerState};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    headers,
    response::IntoResponse,
    routing::{get, post},
    Json, Router, TypedHeader,
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
    _json: Json<LoginInput>,
) -> impl IntoResponse {
    let db: MutexGuard<DatabaseConnection> = server_state.0.db.lock().await;
    let _pool: &PgPool = &db.pool;
    format!("{:?}", _json.0);
}

use crate::{database::DatabaseConnection, modes::ScanningMode, ScannerState};
use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    headers,
    http::Request,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router, TypedHeader,
};
use serde::Deserialize;
use std::{collections::LinkedList, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, TraceLayer},
};
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
        .nest_service("/", serve_dir.clone())
        .route("/ws", get(ws_handler))
        .route("/auth", post(authentication))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .fallback(handler_404); // this fallback shouldn't be necessary but it doesn't hurt to be extra safe
    let routes = routes.with_state(server_state);

    let listener = SocketAddr::from_str(&std::env::var("WEB_LISTEN_URL")?)?;
    axum::Server::bind(&listener)
        .serve(routes.into_make_service())
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
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        println!("pong :3");
    }
    println!("Websocket context {who} destroyed");
}

#[derive(Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[allow(clippy::unused_async, unused)]
async fn authentication(
    server_state: State<ServerState>,
    json: Json<LoginInput>,
) -> impl IntoResponse {
    "hi user"
}

#[allow(clippy::unused_async, unused)]
async fn handler_404(req: Request<Body>) -> Html<&'static str> {
    Html(include_str!("../../web/404.html"))
}

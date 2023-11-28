use crate::{database::DatabaseConnection, modes::ScanningMode, ScannerState};
use axum::{
    body::Body,
    extract::State,
    http::Request,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::{collections::LinkedList, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};
use tokio::sync::Mutex;

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
    let server_state = ServerState {
        db,
        state,
        task_queue,
    };

    let routes = Router::new()
        .route("/", get(root))
        .route("/auth", post(authentication))
        .fallback(handler_404);
    let routes = routes.with_state(server_state);

    let listener = SocketAddr::from_str(&std::env::var("WEB_LISTEN_URL")?)?;
    axum::Server::bind(&listener)
        .serve(routes.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[axum::debug_handler]
#[allow(clippy::unused_async, unused)]
async fn root(server_state: State<ServerState>, req: Request<Body>) -> String {
    format!("req = {req:?}")
}

#[derive(Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[axum::debug_handler]
#[allow(clippy::unused_async, unused)]
async fn authentication(
    server_state: State<ServerState>,
    json: Json<LoginInput>,
) -> impl IntoResponse {
    "hi user"
}

#[allow(clippy::unused_async, unused)]
async fn handler_404(req: Request<Body>) -> Html<&'static str> {
    Html("<h1>404 Error</h1>")
}

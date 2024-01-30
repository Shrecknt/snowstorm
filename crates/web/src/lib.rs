use crate::actions::actions::Action;
use actions::web_actions::web_actions_handler;
use axum::{
    body::{boxed, Body, BoxBody},
    http::{Request, Response, Uri},
    routing::{get, post},
    Router,
};
use database::DatabaseConnection;
use dotenvy_macro::dotenv as var;
use io::modes::ScanningMode;
use io::ScannerState;
use reqwest::StatusCode;
use std::{collections::LinkedList, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower::ServiceExt;
use tower_http::services::ServeDir;
#[cfg(debug_assertions)]
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
#[cfg(debug_assertions)]
use tracing_subscriber::prelude::*;

pub mod actions;
pub mod authentication;
pub mod jwt;

mod api;
mod oauth;
mod ws;

#[derive(Clone)]
pub struct ServerState {
    pub db: DatabaseConnection,
    pub state: Arc<Mutex<ScannerState>>,
    pub mode_queue: Arc<Mutex<LinkedList<(ScanningMode, Duration)>>>,
    pub action_queue: Arc<Mutex<LinkedList<Action>>>,
}

pub async fn start_server(
    db: DatabaseConnection,
    state: Arc<Mutex<ScannerState>>,
    mode_queue: Arc<Mutex<LinkedList<(ScanningMode, Duration)>>>,
    action_queue: Arc<Mutex<LinkedList<Action>>>,
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
        mode_queue,
        action_queue,
    };

    let routes = Router::new()
        .nest_service("/", get(handler))
        .route("/ws", get(ws::ws_handler))
        .route("/auth/login", post(authentication::login))
        .route("/auth/signup", post(authentication::create_account))
        .route("/auth/discord", post(oauth::discord::link_account))
        .route("/auth/forgejo", post(oauth::forgejo::link_account))
        .route("/auth/info", get(authentication::info))
        .route("/actions", post(web_actions_handler))
        .route("/oauth2", get(oauth::discord::oauth2))
        .route("/oauth2_discord", get(oauth::discord::oauth2))
        .route("/oauth2_forgejo", get(oauth::forgejo::oauth2));
    #[cfg(debug_assertions)]
    let routes = routes.layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );
    let routes = routes.with_state(server_state);

    let listener = SocketAddr::from_str(var!("WEB_LISTEN_URL"))?;
    axum::Server::bind(&listener)
        .serve(routes.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

async fn handler(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let res = get_static_file(uri.clone()).await?;
    if res.status() == StatusCode::NOT_FOUND {
        match format!("{}.html?{}", uri.path(), uri.query().unwrap_or("")).parse() {
            Ok(uri_html) => {
                let res = get_static_file(uri_html).await?;
                if res.status() == StatusCode::NOT_FOUND {
                    get_static_file("/404.html".parse().unwrap()).await
                } else {
                    Ok(res)
                }
            }
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid URI".to_string())),
        }
    } else {
        Ok(res)
    }
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();
    let res = match ServiceExt::oneshot(ServeDir::new("web/build"), req).await {
        Ok(res) => res.map(boxed),
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", err),
            ))
        }
    };
    Ok(res)
}

use super::ServerState;
use crate::database::{user::User, DatabaseConnection, DbPush};
use axum::{
    extract::State,
    http::header::{HeaderMap, SET_COOKIE},
    response::{IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use sqlx::PgPool;
use tokio::sync::MutexGuard;

#[derive(Deserialize, Debug, Clone)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

impl LoginInput {
    pub fn is_valid(&self) -> bool {
        const ALLOWED_SYMBOLS: [char; 2] = ['_', '-'];
        (3..=16).contains(&self.username.len())
            && (6..=16).contains(&self.password.len())
            && self
                .username
                .chars()
                .all(|x| x.is_alphanumeric() || ALLOWED_SYMBOLS.contains(&x))
            && self
                .password
                .chars()
                .all(|x| x.is_alphanumeric() || ALLOWED_SYMBOLS.contains(&x))
    }
}

pub async fn login(
    server_state: State<ServerState>,
    credentials: Form<LoginInput>,
) -> impl IntoResponse {
    let db: MutexGuard<DatabaseConnection> = server_state.0.db.lock().await;
    let pool: &PgPool = &db.pool;
    let credentials = credentials.0.clone();

    if !credentials.is_valid() {
        return (HeaderMap::new(), Redirect::to("/login.html?error=1"));
    }

    let existing_account = match sqlx::query_as(
        "SELECT * FROM users WHERE username = $1::TEXT AND password = $2::TEXT",
    )
    .bind(&credentials.username)
    .bind(&credentials.password)
    .fetch_optional(pool)
    .await
    {
        Ok(account) => account,
        Err(_) => return (HeaderMap::new(), Redirect::to("/login.html?error=2")),
    };

    if let Some(user) = existing_account {
        return (get_auth_cookies(&user), Redirect::to("/dashboard.html"));
    }
    (HeaderMap::new(), Redirect::to("/login.html?error=3"))
}

pub async fn create_account(
    server_state: State<ServerState>,
    credentials: Form<LoginInput>,
) -> impl IntoResponse {
    let db: MutexGuard<DatabaseConnection> = server_state.0.db.lock().await;
    let pool: &PgPool = &db.pool;
    let credentials = credentials.0.clone();

    if !credentials.is_valid() {
        return (HeaderMap::new(), Redirect::to("/signup.html?error=1"));
    }

    let existing_account = User::get_username(&credentials.username, pool).await;

    if existing_account.is_some() {
        return (HeaderMap::new(), Redirect::to("/signup.html?error=2"));
    }

    let mut user = User::new(&credentials.username, &credentials.password);
    if user.push(pool).await.is_err() {
        return (HeaderMap::new(), Redirect::to("/signup.html?error=3"));
    };
    (get_auth_cookies(&user), Redirect::to("/dashboard.html"))
}

fn get_auth_cookies(user: &User) -> HeaderMap {
    let auth_cookie = format!("auth_token={}", user.username.clone().unwrap());
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, auth_cookie.parse().unwrap());
    headers
}

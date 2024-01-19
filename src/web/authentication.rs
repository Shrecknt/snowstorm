use super::{jwt::UserSession, ServerState};
use crate::database::{user::User, DbPush};
use axum::{
    extract::State,
    headers,
    http::header::{HeaderMap, SET_COOKIE},
    response::{IntoResponse, Redirect},
    Form, TypedHeader,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

impl LoginInput {
    pub fn is_valid(&self) -> bool {
        const ALLOWED_SYMBOLS: [char; 2] = ['_', '-'];
        (3..=16).contains(&self.username.len())
            && (6..=64).contains(&self.password.len())
            && self
                .username
                .chars()
                .all(|x| x.is_alphanumeric() || ALLOWED_SYMBOLS.contains(&x))
            && self.password.chars().all(|x| (' '..'~').contains(&x))
    }

    pub fn hashed_password(&self) -> String {
        hash(&self.password, DEFAULT_COST).unwrap()
    }
}

pub async fn login(
    server_state: State<ServerState>,
    credentials: Form<LoginInput>,
) -> impl IntoResponse {
    let db = server_state.0.db;
    let pool = &db.pool;
    let credentials = credentials.0.clone();

    if !credentials.is_valid() {
        return (HeaderMap::new(), Redirect::to("/login?error=1"));
    }

    let existing_account: Option<User> =
        match sqlx::query_as("SELECT * FROM users WHERE username ILIKE $1::TEXT")
            .bind(&credentials.username)
            .fetch_optional(pool)
            .await
        {
            Ok(account) => account,
            Err(_) => return (HeaderMap::new(), Redirect::to("/login?error=2")),
        };

    if let Some(user) = existing_account {
        if !verify(credentials.password, &user.password).unwrap() {
            return (HeaderMap::new(), Redirect::to("/login?error=2"));
        }

        return (get_auth_cookies(&user), Redirect::to("/dashboard"));
    }
    (HeaderMap::new(), Redirect::to("/login?error=3"))
}

pub async fn create_account(
    server_state: State<ServerState>,
    credentials: Form<LoginInput>,
) -> impl IntoResponse {
    let db = server_state.0.db;
    let pool = &db.pool;
    let credentials = credentials.0.clone();

    if !credentials.is_valid() {
        return (HeaderMap::new(), Redirect::to("/signup?error=1"));
    }

    let existing_account = User::get_username(&credentials.username, pool).await;

    if existing_account.is_some() {
        return (HeaderMap::new(), Redirect::to("/signup?error=2"));
    }

    let mut user = User::new(&credentials.username, &credentials.hashed_password());
    if user.push(pool).await.is_err() {
        return (HeaderMap::new(), Redirect::to("/signup?error=3"));
    };
    (get_auth_cookies(&user), Redirect::to("/dashboard"))
}

pub fn get_auth_cookies(user: &User) -> HeaderMap {
    let session_cookie = UserSession::new_default(user.id.unwrap()).sign().unwrap();
    let auth_cookie = format!(
        "__Host-Authentication={}; Secure; HttpOnly; Path=/; SameSite=Strict;",
        urlencoding::encode(&session_cookie)
    );
    let mut headers = HeaderMap::new();
    headers.append(SET_COOKIE, auth_cookie.parse().unwrap());
    headers
}

pub async fn info(cookies: Option<TypedHeader<headers::Cookie>>) -> impl IntoResponse {
    match UserSession::from_cookies(&cookies) {
        Some(Ok(session)) => format!("{session:?}"),
        _ => "No session".to_string(),
    }
}

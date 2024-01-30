use crate::{
    authentication::{get_auth_cookies, LoginInput},
    ServerState,
};
use axum::{
    extract::{Query, State},
    headers,
    http::HeaderMap,
    response::{IntoResponse, Redirect},
    Form, TypedHeader,
};
use database::{forgejo_user::ForgejoUserInfo, user::User, DbPush};
use dotenvy_macro::dotenv as var;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    RedirectUrl, StandardTokenResponse, TokenResponse, TokenUrl,
};
use reqwest::header::SET_COOKIE;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
pub struct Oauth2Parameters {
    pub code: String,
}

pub async fn oauth2(
    server_state: State<ServerState>,
    oauth2_parameters: Query<Oauth2Parameters>,
) -> impl IntoResponse {
    match try_oauth(oauth2_parameters.0.clone()).await {
        Ok((_result, mut forgejo_user_info)) => {
            let db = server_state.0.db;
            let pool = &db.pool;

            if let Some(forgejo_user) =
                ForgejoUserInfo::get_forgejo_id(forgejo_user_info.forgejo_id, pool).await
            {
                if let Some(user_id) = forgejo_user.user_id {
                    let user = User::get_id(user_id, pool).await.unwrap();
                    return (get_auth_cookies(&user), Redirect::to("/dashboard"));
                }
            };

            let link_code = Uuid::new_v4().to_string();

            forgejo_user_info.link_code = Some(link_code.clone());
            forgejo_user_info.push(pool).await.unwrap();

            let mut headers = HeaderMap::new();
            headers.append(
                SET_COOKIE,
                format!(
                    "__Host-Forgejo-Link={}; Secure; HttpOnly; Path=/; SameSite=Strict;",
                    urlencoding::encode(&link_code)
                )
                .parse()
                .unwrap(),
            );
            headers.append(
                SET_COOKIE,
                format!(
                    "Forgejo-Id={}; Secure; Path=/; SameSite=Strict;",
                    urlencoding::encode(&forgejo_user_info.forgejo_id.to_string())
                )
                .parse()
                .unwrap(),
            );
            (headers, Redirect::to("/forgejo_login"))
        }
        Err(_) => (HeaderMap::new(), Redirect::to("/login")),
    }
}

pub async fn link_account(
    server_state: State<ServerState>,
    cookies: Option<TypedHeader<headers::Cookie>>,
    credentials: Form<LoginInput>,
) -> impl IntoResponse {
    let db = server_state.0.db;
    let pool = &db.pool;
    let credentials = credentials.0.clone();

    let link_code = if let Some(TypedHeader(cookies)) = &cookies {
        match cookies.get("__Host-Forgejo-Link") {
            Some(code) => code,
            None => return (HeaderMap::new(), Redirect::to("/forgejo_login?error=5")),
        }
    } else {
        return (HeaderMap::new(), Redirect::to("/forgejo_login?error=4"));
    };

    if !credentials.is_valid() {
        return (HeaderMap::new(), Redirect::to("/forgejo_login?error=1"));
    }

    let existing_account = User::get_username(&credentials.username, pool).await;

    if existing_account.is_some() {
        return (HeaderMap::new(), Redirect::to("/forgejo_login?error=2"));
    }

    let mut user = User::new(&credentials.username, &credentials.hashed_password());
    if user.push(pool).await.is_err() {
        return (HeaderMap::new(), Redirect::to("/forgejo_login?error=3"));
    };

    let mut forgejo_user = match ForgejoUserInfo::get_link_code(link_code, pool).await {
        Some(user) => user,
        None => return (HeaderMap::new(), Redirect::to("/forgejo_login?error=6")),
    };
    forgejo_user.link_code = None;
    forgejo_user.user_id = user.id;
    forgejo_user.push(pool).await.unwrap();

    (get_auth_cookies(&user), Redirect::to("/dashboard"))
}

pub const BASE_AUTHORIZE_URI: &str = var!("FORGEJO_BASE_AUTHORIZE_URL");
pub const BASE_TOKEN_URI: &str = var!("FORGEJO_BASE_TOKEN_URI");

async fn try_oauth(
    oauth2_parameters: Oauth2Parameters,
) -> eyre::Result<(
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ForgejoUserInfo,
)> {
    let redirect_uri = var!("FORGEJO_REDIRECT_URI");
    let client_id = var!("FORGEJO_CLIENT_ID");
    let client_secret = var!("FORGEJO_CLIENT_SECRET");
    // let guild_id = var!("FORGEJO_GUILD_ID");

    let client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
        AuthUrl::new(BASE_AUTHORIZE_URI.to_string())?,
        Some(TokenUrl::new(BASE_TOKEN_URI.to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?);

    let (_auth_url, _csrf_token) = client.authorize_url(CsrfToken::new_random).url();

    let token_result = client
        .exchange_code(AuthorizationCode::new(oauth2_parameters.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await?;

    let client = reqwest::Client::new();
    let forgejo_user_info = client
        .get(var!("FORGEJO_USER_API_URI"))
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await?
        .json::<ForgejoUserInfo>()
        .await?;

    Ok((token_result, forgejo_user_info))
}

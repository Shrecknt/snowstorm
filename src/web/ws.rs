use super::ServerState;
use crate::{
    database::user::User,
    web::{
        actions::{actions_handler, WebActions},
        jwt::UserSession,
    },
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    headers,
    response::IntoResponse,
    TypedHeader,
};
use serde_json::json;
use std::net::SocketAddr;

#[allow(clippy::unused_async)]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    cookies: Option<TypedHeader<headers::Cookie>>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    server_state: State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user = match UserSession::from_cookies(&cookies) {
        Some(Ok(session)) => {
            if session.is_valid() {
                Some(session.user(&server_state.db.pool).await)
            } else {
                None
            }
        }
        _ => None,
    };

    println!("user: {user:?}");

    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, server_state, user))
}

async fn handle_socket(
    mut socket: WebSocket,
    who: SocketAddr,
    server_state: State<ServerState>,
    user: Option<User>,
) {
    let _ = socket.send(Message::Ping(vec![1, 2, 3])).await;

    while let Some(message) = socket.recv().await {
        match message {
            Ok(message) => {
                let text = match &message {
                    Message::Text(text) => text.trim(),
                    Message::Close(_) => break,
                    _ => continue,
                };
                if text.is_empty() {
                    continue;
                }

                let action: WebActions = match serde_json::from_str(text) {
                    Ok(action) => action,
                    Err(err) => {
                        let _ = socket
                            .send(Message::Text(
                                json!({"success": false, "msg": err.to_string()}).to_string(),
                            ))
                            .await;
                        continue;
                    }
                };

                if let Some(user) = &user {
                    if user.permission_level >= 1 {
                        let result = actions_handler(action, &server_state).await;
                        let _ = socket.send(Message::Text(result.to_string())).await;
                    } else {
                        let _ = socket
                            .send(Message::Text(
                                json!({"success": false, "msg": "insufficient permissions"})
                                    .to_string(),
                            ))
                            .await;
                    }
                } else {
                    let _ = socket
                        .send(Message::Text(
                            json!({"success": false, "msg": "authentication error"}).to_string(),
                        ))
                        .await;
                }
            }
            Err(err) => {
                println!("err: {}", err);
                break;
            }
        }
    }

    println!("Websocket context {who} destroyed");
}

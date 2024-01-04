use crate::web::jwt::UserSession;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, WebSocketUpgrade,
    },
    headers,
    response::IntoResponse,
    TypedHeader,
};
use simd_json::owned::Value;
use std::net::SocketAddr;

#[allow(clippy::unused_async)]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    cookies: Option<TypedHeader<headers::Cookie>>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user = match UserSession::from_cookies(&cookies) {
        Some(Ok(session)) => {
            if session.is_valid() {
                Some(session)
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

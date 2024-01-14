use super::{jwt::UserSession, ServerState};
use crate::Action;
use axum::{extract::State, headers, response::IntoResponse, Json, TypedHeader};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebActions {
    QueueAction { action: Action },
    GetModesQueue {},
}

pub async fn web_actions_handler(
    server_state: State<ServerState>,
    cookies: Option<TypedHeader<headers::Cookie>>,
    action: Json<WebActions>,
) -> impl IntoResponse {
    let user = if let Some(Ok(session)) = UserSession::from_cookies(&cookies) {
        if session.is_valid() {
            Some(session.user(&server_state.db.pool).await)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(user) = user {
        if user.permission_level >= 1 {
            Json(actions_handler(action.0, &server_state).await)
        } else {
            Json(json!({"success": false, "msg": "insufficient permissions"}))
        }
    } else {
        Json(json!({"success": false, "msg": "authentication failure"}))
    }
}

pub async fn actions_handler(action: WebActions, server_state: &State<ServerState>) -> Value {
    match action {
        WebActions::QueueAction { action } => {
            server_state.action_queue.lock().await.push_back(action);
            json!({"success": true, "msg": ""})
        }
        WebActions::GetModesQueue {} => {
            let queue = server_state.mode_queue.lock().await;
            let queue: Vec<_> = queue.iter().collect();
            json!({"success": true, "msg": "", "queue": queue})
        }
        #[allow(unreachable_patterns)]
        _ => json!({"success": false, "msg": "unhandled action type"}),
    }
}

use crate::player::PlayerInfo;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Autocomplete {
    Username { username: String },
    Uuid { uuid: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AutocompleteResults {
    Username { players: Vec<(String, Uuid)> },
    Uuid { players: Vec<(String, Uuid)> },
}

impl Autocomplete {
    pub async fn autocomplete(&self, pool: &PgPool) -> Value {
        match self {
            Autocomplete::Username { username } => {
                json!({"success": true, "msg": "", "data": {"type": "autocomplete", "data": PlayerInfo::autocomplete_username(username, pool).await}})
            }
            Autocomplete::Uuid { .. } => {
                json!({"success": true, "msg": "", "data": {"type": "autocomplete", "data": []}})
            }
        }
    }
}

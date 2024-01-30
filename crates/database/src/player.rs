use super::{autocomplete::AutocompleteResults, DbPush};
use azalea_protocol::packets::status::clientbound_status_response_packet::ClientboundStatusResponsePacket;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool, Row};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct PlayerInfo {
    #[serde(skip)]
    pub id: Option<i64>,
    pub uuid: Uuid,
    pub username: String,
    #[serde(skip)]
    pub server: Option<i64>,
}

impl PlayerInfo {
    pub fn new(username: String, uuid: Uuid) -> Self {
        Self {
            id: None,
            uuid,
            username,
            server: None,
        }
    }

    pub async fn autocomplete_username(username: &str, pool: &PgPool) -> AutocompleteResults {
        let players = sqlx::query_as(
            "SELECT username, uuid FROM players WHERE username ILIKE '%' || $1::TEXT || '%' LIMIT 16",
        )
        .bind(username)
        .fetch_all(pool)
        .await
        .unwrap();
        AutocompleteResults::Username { players }
    }

    pub async fn from_username(username: &str, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM players WHERE username = $1::TEXT")
            .bind(username)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub async fn from_uuid(uuid: Uuid, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM players WHERE uuid = $1::UUID")
            .bind(uuid)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub fn from_azalea(value: &ClientboundStatusResponsePacket) -> Vec<Self> {
        let mut players = Vec::with_capacity(value.players.sample.len());
        for player in &value.players.sample {
            let Ok(uuid) = Uuid::parse_str(&player.id) else {
                continue;
            };
            players.push(Self {
                id: None,
                username: player.name.clone(),
                uuid,
                server: None,
            });
        }
        players
    }
}

impl DbPush for PlayerInfo {
    async fn push(&mut self, pool: &sqlx::PgPool) -> Result<(), eyre::Report> {
        const QUERY: &str = "INSERT INTO players (
                uuid,
                username
            ) VALUES (
                $2::UUID,
                $3::TEXT
            )
            ON CONFLICT (uuid, username) DO NOTHING
            RETURNING id";
        let new_id: i64 = sqlx::query(QUERY)
            .bind(self.id)
            .bind(self.uuid)
            .bind(self.username.clone())
            .fetch_one(pool)
            .await?
            .get("id");
        self.id = Some(new_id);

        if let Some(server_id) = self.server {
            sqlx::query(
                "INSERT INTO join_table (
                server_id,
                player_id,
                discovered,
                last_seen
            ) VALUES (
                $1::BIGINT,
                $2::BIGINT,
                $3::BIGINT,
                $3::BIGINT
            ) ON CONFLICT (server_id, player_id) DO UPDATE SET
                last_seen = excluded.last_seen
            ",
            )
            .bind(server_id)
            .bind(new_id)
            .bind(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as i64,
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}

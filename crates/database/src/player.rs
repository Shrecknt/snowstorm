use super::autocomplete::AutocompleteResults;
use azalea_protocol::packets::status::clientbound_status_response_packet::ClientboundStatusResponsePacket;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct PlayerInfo {
    #[serde(skip)]
    pub id: Option<i64>,
    pub uuid: Uuid,
    pub username: String,
    pub java_account: Option<bool>,
    pub bedrock_account: Option<bool>,
}

impl PlayerInfo {
    pub fn new_unchecked(username: impl ToString, uuid: impl Into<Uuid>) -> Self {
        let username = username.to_string();
        let uuid = uuid.into();

        Self {
            id: None,
            uuid,
            username,
            java_account: None,
            bedrock_account: None,
        }
    }

    pub async fn new(username: impl ToString, uuid: impl Into<Uuid>) -> Self {
        let username = username.to_string();
        let uuid = uuid.into();

        let java_account = {
            let account_data = mowojang::check_uuid(uuid).await;
            if let Some(account_data) = account_data {
                Some(account_data.name.eq_ignore_ascii_case(&username))
            } else {
                None
            }
        };

        Self {
            id: None,
            uuid,
            username,
            java_account,
            bedrock_account: None,
        }
    }

    pub async fn autocomplete_username(username: &str, pool: &PgPool) -> AutocompleteResults {
        let players = sqlx::query_as(
            "SELECT id, uuid, username FROM players WHERE username ILIKE '%' || $1::TEXT || '%' LIMIT 16",
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

    pub async fn from_id(id: i64, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM players WHERE id = $1::BIGINT")
            .bind(id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub async fn from_azalea(value: &ClientboundStatusResponsePacket) -> Vec<Self> {
        let mut players = Vec::with_capacity(value.players.sample.len());
        for player in &value.players.sample {
            let Ok(uuid) = Uuid::parse_str(&player.id) else {
                continue;
            };
            players.push(Self::new(&player.name, uuid).await);
        }
        players
    }
}

impl PlayerInfo {
    pub async fn push(&mut self, server_id: i64, pool: &sqlx::PgPool) -> Result<(), eyre::Report> {
        const QUERY: &str = "INSERT INTO players (
                uuid,
                username,
                java_account,
                bedrock_account,
            ) VALUES (
                $2::UUID,
                $3::TEXT,
                $4::BOOLEAN,
                $5::BOOLEAN
            )
            ON CONFLICT (uuid, username) DO UPDATE SET
                java_account = excluded.java_account,
                bedrock_account = excluded.bedrock_account
            RETURNING id";
        let new_id: i64 = sqlx::query(QUERY)
            .bind(self.id)
            .bind(self.uuid)
            .bind(&self.username)
            .bind(self.java_account)
            .bind(self.bedrock_account)
            .fetch_one(pool)
            .await?
            .get("id");
        self.id = Some(new_id);

        sqlx::query(
            "INSERT INTO join_servers_players (
                server_id,
                player_id
            ) VALUES (
                $1::BIGINT,
                $2::BIGINT
            ) ON CONFLICT (server_id, player_id) DO UPDATE SET
                last_seen = EXTRACT(epoch from now())
            ",
        )
        .bind(server_id)
        .bind(new_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

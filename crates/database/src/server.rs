use super::DbPush;
use azalea_protocol::packets::status::clientbound_status_response_packet::ClientboundStatusResponsePacket;
use serde::Serialize;
use sqlx::{PgPool, Row};
use std::net::Ipv4Addr;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PingResult {
    pub id: Option<i64>,
    // host info
    pub ip: i32,
    pub port: i16,
    // ping results
    pub version_name: Option<String>,
    pub version_protocol: Option<i32>,
    pub max_players: Option<i32>,
    pub online_players: Option<i32>,
    pub description: Option<String>,
    pub enforces_secure_chat: Option<bool>,
    pub previews_chat: Option<bool>,
    // timestamps
    pub discovered: i64,
    pub last_seen: i64,
}

impl PingResult {
    pub fn ip(&self) -> Ipv4Addr {
        (self.ip as u32).into()
    }

    pub fn port(&self) -> u16 {
        self.port as u16
    }

    pub fn set_ip(&mut self, ip: Ipv4Addr) {
        self.ip = u32::from(ip) as i32;
    }

    pub fn none(ip: Ipv4Addr, port: u16) -> Self {
        Self {
            id: None,
            ip: u32::from(ip) as i32,
            port: port as i16,
            version_name: None,
            version_protocol: None,
            max_players: None,
            online_players: None,
            description: None,
            enforces_secure_chat: None,
            previews_chat: None,
            discovered: 0,
            last_seen: 0,
        }
    }

    pub fn from_azalea(ip: Ipv4Addr, port: u16, value: &ClientboundStatusResponsePacket) -> Self {
        Self {
            id: None,
            ip: u32::from(ip) as i32,
            port: port as i16,
            version_name: Some(value.version.name.clone()),
            version_protocol: Some(value.version.protocol),
            max_players: Some(value.players.max),
            online_players: Some(value.players.online),
            description: Some(value.description.to_string()),
            enforces_secure_chat: value.enforces_secure_chat,
            previews_chat: None,
            discovered: 0,
            last_seen: 0,
        }
    }

    pub async fn from_player_id(player_id: i64, pool: &PgPool) -> Vec<Self> {
        const QUERY_STRING: &str = "
        WITH joins AS (
            SELECT server_id FROM join_table WHERE player_id = $1::BIGINT
        ) SELECT * FROM servers WHERE id IN (SELECT server_id FROM joins);
        ";
        sqlx::query_as(QUERY_STRING)
            .bind(player_id)
            .fetch_all(pool)
            .await
            .unwrap()
    }

    pub async fn from_ip_port(ip: &Ipv4Addr, port: u16, pool: &PgPool) -> Option<Self> {
        const QUERY_STRING: &str = "
        SELECT * FROM servers WHERE ip = $1::INT AND port = $2::SMALLINT;
        ";
        sqlx::query_as(QUERY_STRING)
            .bind(u32::from(*ip) as i32)
            .bind(port as i16)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub async fn from_id(id: i64, pool: &PgPool) -> Option<Self> {
        const QUERY_STRING: &str = "
        SELECT * FROM servers WHERE id = $1::BIGINT;
        ";
        sqlx::query_as(QUERY_STRING)
            .bind(id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }
}

impl DbPush for PingResult {
    async fn push(&mut self, pool: &sqlx::PgPool) -> Result<(), eyre::Report> {
        let query = "INSERT INTO servers (
                    ip,
                    port,
                    version_name,
                    version_protocol,
                    max_players,
                    online_players,
                    description,
                    enforces_secure_chat,
                    previews_chat
                ) VALUES (
                    $2::INT,
                    $3::SMALLINT,
                    $4::TEXT,
                    $5::INT,
                    $6::INT,
                    $7::INT,
                    $8::TEXT,
                    $9::BOOLEAN,
                    $10::BOOLEAN
                ) ON CONFLICT (ip, port) DO UPDATE SET
                    version_name = excluded.version_name,
                    version_protocol = excluded.version_protocol,
                    max_players = excluded.max_players,
                    online_players = excluded.max_players,
                    description = excluded.description,
                    enforces_secure_chat = excluded.enforces_secure_chat,
                    previews_chat = excluded.previews_chat,
                    last_seen = excluded.last_seen
                RETURNING id";
        let new_id: i64 = sqlx::query(query)
            .bind(self.id)
            .bind(self.ip as i32)
            .bind(self.port as i16)
            .bind(self.version_name.clone())
            .bind(self.version_protocol)
            .bind(self.max_players)
            .bind(self.online_players)
            .bind(self.description.clone())
            .bind(self.enforces_secure_chat)
            .bind(self.previews_chat)
            .fetch_one(pool)
            .await?
            .get("id");
        self.id = Some(new_id);
        Ok(())
    }
}

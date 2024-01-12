use super::DbPush;
use azalea_protocol::packets::status::clientbound_status_response_packet::ClientboundStatusResponsePacket;
use serde::Serialize;
use sqlx::Row;
use std::net::Ipv4Addr;

#[derive(Debug, Serialize)]
pub struct PingResult {
    pub id: Option<i64>,
    // host info
    pub ip: u32,
    pub port: u16,
    // ping results
    pub version_name: Option<String>,
    pub version_protocol: Option<i32>,
    pub max_players: Option<i32>,
    pub online_players: Option<i32>,
    pub description: Option<String>,
    pub enforces_secure_chat: Option<bool>,
    pub previews_chat: Option<bool>,
}

impl PingResult {
    pub fn get_ip(&self) -> Ipv4Addr {
        self.ip.into()
    }

    pub fn set_ip(&mut self, ip: Ipv4Addr) {
        self.ip = ip.into();
    }

    pub fn none(ip: Ipv4Addr, port: u16) -> Self {
        Self {
            id: None,
            ip: ip.into(),
            port,
            version_name: None,
            version_protocol: None,
            max_players: None,
            online_players: None,
            description: None,
            enforces_secure_chat: None,
            previews_chat: None,
        }
    }

    pub fn from_azalea(ip: Ipv4Addr, port: u16, value: &ClientboundStatusResponsePacket) -> Self {
        Self {
            id: None,
            ip: ip.into(),
            port,
            version_name: Some(value.version.name.clone()),
            version_protocol: Some(value.version.protocol),
            max_players: Some(value.players.max),
            online_players: Some(value.players.online),
            description: Some(value.description.to_string()),
            enforces_secure_chat: value.enforces_secure_chat,
            previews_chat: None,
        }
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
                    previews_chat = excluded.previews_chat
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

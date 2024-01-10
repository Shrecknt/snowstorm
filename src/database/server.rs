use azalea_protocol::packets::status::clientbound_status_response_packet::ClientboundStatusResponsePacket;
use serde::Serialize;
use std::net::Ipv4Addr;

#[derive(Debug, Serialize)]
pub struct PingResult {
    // host info
    pub ip: Ipv4Addr,
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
    pub fn none(ip: Ipv4Addr, port: u16) -> Self {
        Self {
            ip,
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
            ip,
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

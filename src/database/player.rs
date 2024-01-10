use azalea_protocol::packets::status::clientbound_status_response_packet::ClientboundStatusResponsePacket;
use serde::Serialize;
use std::net::Ipv4Addr;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerInfo {
    pub name: String,
    pub uuid: Uuid,
    #[serde(skip)]
    pub server: Option<(Ipv4Addr, u16)>,
}

impl PlayerInfo {
    pub fn new(name: String, uuid: Uuid) -> Self {
        Self {
            name,
            uuid,
            server: None,
        }
    }

    pub fn from_username(name: String) -> Self {
        #[allow(unreachable_code)]
        Self {
            name,
            uuid: todo!(),
            server: None,
        }
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        #[allow(unreachable_code)]
        Self {
            uuid,
            name: todo!(),
            server: None,
        }
    }

    pub fn from_azalea(
        ip: Ipv4Addr,
        port: u16,
        value: &ClientboundStatusResponsePacket,
    ) -> Vec<Self> {
        let mut players = Vec::with_capacity(value.players.sample.len());
        for player in &value.players.sample {
            let Ok(uuid) = Uuid::parse_str(&player.id) else {
                continue;
            };
            players.push(Self {
                name: player.name.clone(),
                uuid,
                server: Some((ip, port)),
            });
        }
        players
    }
}

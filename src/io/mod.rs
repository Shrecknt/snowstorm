use std::net::Ipv4Addr;
use uuid::Uuid;

pub mod database;
pub mod network;

pub trait Io {
    fn ping(
        &self,
        addr: Ipv4Addr,
        port: u16,
    ) -> impl std::future::Future<Output = eyre::Result<()>> + Send;

    fn legacy_ping(
        &self,
        addr: Ipv4Addr,
        port: u16,
    ) -> impl std::future::Future<Output = eyre::Result<()>> + Send;
}

#[derive(Debug)]
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerInfo {
    pub name: String,
    pub uuid: Uuid,
}

impl PlayerInfo {
    pub fn new(name: String, uuid: Uuid) -> Self {
        Self { name, uuid }
    }

    pub fn from_username(name: String) -> Self {
        #[allow(unreachable_code)]
        Self {
            name,
            uuid: todo!(),
        }
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        #[allow(unreachable_code)]
        Self {
            uuid,
            name: todo!(),
        }
    }
}

use std::hash::{DefaultHasher, Hash, Hasher};

pub mod database;
pub mod network;
pub mod pnet;
pub mod proxy;

lazy_static::lazy_static! {
    static ref COOKIE_SEED: u64 = rand::random();
}

pub trait Io {
    fn ping(
        &mut self,
        addr: std::net::SocketAddrV4,
    ) -> impl std::future::Future<Output = eyre::Result<()>> + Send;

    fn legacy_ping(
        &mut self,
        addr: std::net::SocketAddrV4,
    ) -> impl std::future::Future<Output = eyre::Result<()>> + Send;
}

pub fn cookie(address: &std::net::SocketAddrV4, seed: u64) -> u32 {
    let mut hasher = DefaultHasher::new();
    (*address.ip(), address.port(), seed).hash(&mut hasher);
    hasher.finish() as u32
}

#[derive(Default)]
pub struct ScannerState {
    pub discovered: u64,
}

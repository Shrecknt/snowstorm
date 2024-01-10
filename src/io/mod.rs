pub mod database;
pub mod network;
pub mod pnet;
pub mod proxy;

pub trait Io {
    fn ping(
        &self,
        addr: std::net::Ipv4Addr,
        port: u16,
    ) -> impl std::future::Future<Output = eyre::Result<()>> + Send;

    fn legacy_ping(
        &self,
        addr: std::net::Ipv4Addr,
        port: u16,
    ) -> impl std::future::Future<Output = eyre::Result<()>> + Send;
}

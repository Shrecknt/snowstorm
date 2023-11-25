use super::Io;
use std::net::Ipv4Addr;

pub struct NetworkScanner {}

impl Io for NetworkScanner {
    async fn ping(&self, _addr: Ipv4Addr, _port: u16) -> Result<(), eyre::Report> {
        todo!()
    }

    async fn legacy_ping(&self, _addr: Ipv4Addr, _port: u16) -> Result<(), eyre::Report> {
        todo!()
    }
}

use super::Io;
use crate::{
    database::{player::PlayerInfo, server::PingResult},
    ScannerState,
};
use std::{
    net::Ipv4Addr,
    sync::{mpsc::Sender, Arc},
};
use tokio::sync::Mutex;

pub struct PnetScanner {
    pub state: Arc<Mutex<ScannerState>>,
    pub sender: Sender<(PingResult, Vec<PlayerInfo>)>,
}

impl Io for PnetScanner {
    async fn ping(&self, _addr: Ipv4Addr, _port: u16) -> Result<(), eyre::Report> {
        todo!()
    }

    async fn legacy_ping(&self, _addr: Ipv4Addr, _port: u16) -> Result<(), eyre::Report> {
        todo!()
    }
}

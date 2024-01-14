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

mod receive;

pub struct PnetScanner {
    pub state: Arc<Mutex<ScannerState>>,
}

impl PnetScanner {
    pub fn new(
        state: Arc<Mutex<ScannerState>>,
        sender: Sender<(PingResult, Vec<PlayerInfo>)>,
    ) -> Self {
        tokio::spawn(async move { receive::start_server(sender).await });
        Self { state }
    }
}

impl Io for PnetScanner {
    async fn ping(&self, _addr: Ipv4Addr, _port: u16) -> Result<(), eyre::Report> {
        todo!()
    }

    async fn legacy_ping(&self, _addr: Ipv4Addr, _port: u16) -> Result<(), eyre::Report> {
        todo!()
    }
}

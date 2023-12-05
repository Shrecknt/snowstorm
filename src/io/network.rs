use super::{Io, PingResult, PlayerInfo};
use crate::ScannerState;
use std::{
    net::Ipv4Addr,
    sync::{mpsc::Sender, Arc},
};
use tokio::sync::Mutex;

pub struct NetworkScanner {
    pub state: Arc<Mutex<ScannerState>>,
    pub sender: Sender<(PingResult, Vec<PlayerInfo>)>,
}

impl NetworkScanner {
    pub fn listen(
        state: Arc<Mutex<ScannerState>>,
        sender: Sender<(PingResult, Vec<PlayerInfo>)>,
    ) -> Self {
        let res = Self { state, sender };

        // TODO: use pnet to listen for pings
        res.sender
            .send((PingResult::none(Ipv4Addr::from(0), 0), vec![]))
            .expect("Unable to send");

        res
    }
}

impl Io for NetworkScanner {
    async fn ping(&self, _addr: Ipv4Addr, _port: u16) -> Result<(), eyre::Report> {
        todo!()
    }

    async fn legacy_ping(&self, _addr: Ipv4Addr, _port: u16) -> Result<(), eyre::Report> {
        todo!()
    }
}

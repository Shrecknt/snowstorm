use super::Io;
use crate::{
    database::{player::PlayerInfo, server::PingResult},
    ScannerState,
};
use dotenvy_macro::dotenv as var;
use std::{
    collections::BTreeSet,
    net::Ipv4Addr,
    sync::{mpsc::Sender, Arc},
};
use tokio::sync::Mutex;

pub struct DatabaseScanner {
    pub state: Arc<Mutex<ScannerState>>,
    pub sender: Sender<(PingResult, Vec<PlayerInfo>)>,
    pub data: BTreeSet<(Ipv4Addr, u16)>,
}

impl DatabaseScanner {
    pub fn new(
        state: Arc<Mutex<ScannerState>>,
        sender: Sender<(PingResult, Vec<PlayerInfo>)>,
    ) -> Self {
        let data = csv::Reader::from_path(var!("TESTING_DATA"))
            .unwrap()
            .records()
            .map(|item| {
                let item = item.unwrap();
                (
                    Ipv4Addr::from(item[0].parse::<u32>().unwrap()),
                    item[1].parse::<u16>().unwrap(),
                )
            })
            .collect();

        Self {
            state,
            sender,
            data,
        }
    }
}

impl Io for DatabaseScanner {
    async fn ping(&self, addr: Ipv4Addr, port: u16) -> eyre::Result<()> {
        if self.data.contains(&(addr, port)) {
            self.state.lock().await.discovered += 1;
            self.sender
                .send((PingResult::none(addr, port), vec![]))
                .expect("Unable to send ping result");
        }
        Ok(())
    }

    async fn legacy_ping(&self, addr: Ipv4Addr, port: u16) -> eyre::Result<()> {
        if self.data.contains(&(addr, port)) {
            self.state.lock().await.discovered += 1;
            self.sender
                .send((PingResult::none(addr, port), vec![]))
                .expect("Unable to send ping result");
        }
        Ok(())
    }
}

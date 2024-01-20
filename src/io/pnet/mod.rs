use self::constants::C2SSequenceNumbers;
use super::{cookie, Io, COOKIE_SEED};
use crate::{
    database::{player::PlayerInfo, server::PingResult},
    util::net::{
        source_port::SourcePort,
        tcp::{StatelessTcp, StatelessTcpWriteHalf},
    },
    ScannerState,
};
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::{mpsc::Sender, Arc},
};
use tokio::sync::Mutex;

pub mod constants;

mod receive;

pub struct PnetScanner {
    pub state: Arc<Mutex<ScannerState>>,
    pub syn_writer: StatelessTcpWriteHalf,
    pub source_port: SourcePort,
}

impl PnetScanner {
    pub fn new(
        state: Arc<Mutex<ScannerState>>,
        sender: Sender<(PingResult, Vec<PlayerInfo>)>,
    ) -> Self {
        let source_port = SourcePort::Number(61000);
        let socket = StatelessTcp::new(source_port);
        let syn_writer = socket.write.clone();
        tokio::spawn(async move { receive::start_server(socket, sender, source_port).await });
        Self {
            state,
            syn_writer,
            source_port,
        }
    }
}

impl Io for PnetScanner {
    async fn ping(&mut self, addr: Ipv4Addr, port: u16) -> Result<(), eyre::Report> {
        let addr = SocketAddrV4::new(addr, port);
        let addr_cookie = cookie(&addr, *COOKIE_SEED);
        self.syn_writer.send_syn(
            addr,
            self.source_port.pick(addr_cookie),
            addr_cookie + C2SSequenceNumbers::SlpSyn,
        );
        Ok(())
    }

    async fn legacy_ping(&mut self, addr: Ipv4Addr, port: u16) -> Result<(), eyre::Report> {
        let addr = SocketAddrV4::new(addr, port);
        let addr_cookie = cookie(&addr, *COOKIE_SEED);
        self.syn_writer.send_syn(
            addr,
            self.source_port.pick(addr_cookie),
            addr_cookie + C2SSequenceNumbers::LegacySyn,
        );
        Ok(())
    }
}

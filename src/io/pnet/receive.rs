use super::constants::{C2SAcknowledgementNumbers, C2SSequenceNumbers};
use crate::{
    database::{player::PlayerInfo, server::PingResult},
    io::{cookie, COOKIE_SEED},
    util::net::{source_port::SourcePort, tcp::StatelessTcp},
};
use std::{net::SocketAddrV4, sync::mpsc::Sender};

lazy_static::lazy_static! {
    pub static ref SLP_PING_PACKET: Vec<u8> = {
        todo!()
    };
    pub static ref LEGACY_PING_PACKET: Vec<u8> = {
        todo!()
    };
}

pub async fn start_server(
    mut socket: StatelessTcp,
    _sender: Sender<(PingResult, Vec<PlayerInfo>)>,
    source_port: SourcePort,
) {
    while let Some((ip, tcp)) = socket.read.recv() {
        let source_addr = SocketAddrV4::new(ip.source, tcp.source);
        let cookie = cookie(&source_addr, *COOKIE_SEED);
        let sequence = tcp.sequence - cookie;
        match sequence {
            // syn + ack
            0x00000001 => {
                if cookie + C2SAcknowledgementNumbers::SlpAck != tcp.acknowledgement {
                    return;
                }
                let payload = &*SLP_PING_PACKET;
                socket.write.send_data(
                    source_addr,
                    source_port.pick(cookie),
                    cookie + C2SSequenceNumbers::SlpAck,
                    cookie + C2SAcknowledgementNumbers::SlpAck,
                    payload,
                );
            }
            0x10000001 => todo!(),
            _ => {}
        }
    }
}

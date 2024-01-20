use super::constants::{C2SAcknowledgementNumbers, C2SSequenceNumbers, S2CAcknowledgementNumbers};
use crate::{
    database::{player::PlayerInfo, server::PingResult},
    io::{cookie, COOKIE_SEED},
    util::net::tcp::StatelessTcp,
};
use azalea_protocol::{packets::status::ClientboundStatusPacket, read::deserialize_packet};
use pnet::packet::tcp::TcpFlags;
use std::{collections::HashMap, io::Cursor, net::SocketAddrV4, sync::mpsc::Sender};

#[rustfmt::skip]
pub mod const_packets {
    pub const SLP_PING_PACKET: [u8; 32] = {
        [
            0x1d,                   // packet length = 29
            0x00,                   // packet id = set protocol
            0xfd, 0x05,             // protocol version = 765
            0x16,                   // hostname length = 22
            0x73, 0x6e, 0x6f, 0x77, // hostname = snowstorm.shrecked.dev
            0x73, 0x74, 0x6f, 0x72, // ^
            0x6d, 0x2e, 0x73, 0x68, // ^
            0x72, 0x65, 0x63, 0x6b, // ^
            0x65, 0x64, 0x2e, 0x64, // ^
            0x65, 0x76,             // ^
            0xa4, 0x55,             // port = 42069
            0x01,                   // next state = status request
            0x01, 0x00,             // status request
        ]
    };
    pub const LEGACY_PING_PACKET: [u8; 3] = {
        [
            0xfe, 0x01, 0xfa,       // legacy ping
        ]
    };
}
use const_packets::*;

const SYN_ACK: u8 = TcpFlags::SYN | TcpFlags::ACK;
const FIN_ACK: u8 = TcpFlags::FIN | TcpFlags::ACK;

pub async fn start_server(mut socket: StatelessTcp, sender: Sender<(PingResult, Vec<PlayerInfo>)>) {
    let mut awaiting_data_map: HashMap<SocketAddrV4, Vec<u8>> = HashMap::new();

    while let Some((ip, mut tcp)) = socket.read.recv() {
        let source_addr = SocketAddrV4::new(ip.source, tcp.source);
        let cookie = cookie(&source_addr, *COOKIE_SEED);
        let sequence = tcp.sequence - cookie;
        match sequence {
            // syn + ack
            0x00000000 => {
                if tcp.flags & SYN_ACK != SYN_ACK {
                    #[cfg(debug_assertions)]
                    println!(
                        "expected flags = {SYN_ACK}\ngot flags = {} ({})",
                        tcp.flags & SYN_ACK,
                        tcp.flags
                    );
                    return;
                }
                if cookie + S2CAcknowledgementNumbers::SlpSynAck != tcp.acknowledgement {
                    #[cfg(debug_assertions)]
                    println!(
                        "expected ack = {}\ngot ack = {}",
                        cookie + S2CAcknowledgementNumbers::SlpSynAck,
                        tcp.acknowledgement
                    );
                    return;
                }
                socket.write.send_ack(
                    source_addr,
                    tcp.destination,
                    cookie + C2SSequenceNumbers::SlpSynAck,
                    cookie + C2SAcknowledgementNumbers::SlpAck,
                );
                socket.write.send_data(
                    source_addr,
                    tcp.destination,
                    cookie + C2SSequenceNumbers::SlpSynAck,
                    cookie + C2SAcknowledgementNumbers::SlpAck,
                    &SLP_PING_PACKET,
                );
            }
            // payload
            0x00000001..=0x00000009 => {
                if cookie + S2CAcknowledgementNumbers::SlpResponsePayload != tcp.acknowledgement {
                    #[cfg(debug_assertions)]
                    println!(
                        "expected ack = {}\ngot ack = {}",
                        cookie + S2CAcknowledgementNumbers::SlpResponsePayload,
                        tcp.acknowledgement
                    );
                    return;
                }
                let buffer = match awaiting_data_map.get_mut(&source_addr) {
                    Some(buffer) => buffer,
                    None => {
                        let buffer = Vec::with_capacity(512);
                        awaiting_data_map.insert(source_addr, buffer);
                        awaiting_data_map.get_mut(&source_addr).unwrap()
                    }
                };
                let payload_len = tcp.payload.len();
                buffer.append(&mut tcp.payload);
                if tcp.flags & TcpFlags::FIN == TcpFlags::FIN {
                    socket.write.send_fin(
                        source_addr,
                        tcp.destination,
                        tcp.acknowledgement,
                        tcp.sequence + payload_len as u32,
                    );
                    let mut buffer = Cursor::new(buffer.as_slice());
                    let ping_response =
                        match deserialize_packet::<ClientboundStatusPacket>(&mut buffer) {
                            Ok(ClientboundStatusPacket::StatusResponse(ping_response)) => {
                                ping_response
                            }
                            _ => return,
                        };
                    let ping_result =
                        PingResult::from_azalea(ip.source, tcp.source, &ping_response);
                    let player_info = PlayerInfo::from_azalea(&ping_response);
                    let _ = sender.send((ping_result, player_info));
                    awaiting_data_map.remove(&source_addr);
                }
            }
            0x00000010 => {
                #[cfg(debug_assertions)]
                println!("Connection from {source_addr} cut off after too many packets");
                socket.write.send_fin(
                    source_addr,
                    tcp.destination,
                    tcp.acknowledgement,
                    tcp.sequence + 1,
                );
                awaiting_data_map.remove(&source_addr);
            }
            // syn + ack
            0x10000000 => {
                if cookie + S2CAcknowledgementNumbers::LegacySynAck != tcp.acknowledgement {
                    return;
                }
                socket.write.send_ack(
                    source_addr,
                    tcp.destination,
                    cookie + C2SSequenceNumbers::LegacySynAck,
                    cookie + C2SAcknowledgementNumbers::LegacyAck,
                );
                socket.write.send_data(
                    source_addr,
                    tcp.destination,
                    cookie + C2SSequenceNumbers::LegacySynAck,
                    cookie + C2SAcknowledgementNumbers::LegacyAck,
                    &LEGACY_PING_PACKET,
                );
            }
            _ => {
                if tcp.flags & FIN_ACK == FIN_ACK {
                    socket.write.send_ack(
                        source_addr,
                        tcp.destination,
                        tcp.acknowledgement,
                        tcp.sequence + 1,
                    );
                } else {
                    socket.write.send_fin(
                        source_addr,
                        tcp.destination,
                        tcp.acknowledgement,
                        tcp.sequence + 1,
                    );
                }
                awaiting_data_map.remove(&source_addr);
            }
        }
    }
}

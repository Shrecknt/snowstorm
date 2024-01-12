use super::Io;
use crate::{
    database::{player::PlayerInfo, server::PingResult},
    ScannerState,
};
use azalea_protocol::{
    connect::RawReadConnection,
    packets::{
        handshaking::client_intention_packet::ClientIntentionPacket,
        status::{
            serverbound_status_request_packet::ServerboundStatusRequestPacket,
            ClientboundStatusPacket,
        },
        ConnectionProtocol,
    },
    read::deserialize_packet,
    write::write_packet,
};
use bytes::BytesMut;
use std::{
    io::Cursor,
    net::{Ipv4Addr, SocketAddr},
    sync::{mpsc::Sender, Arc},
};
use tokio::{net::TcpStream, sync::Mutex};

pub struct NetworkScanner {
    pub state: Arc<Mutex<ScannerState>>,
    pub sender: Sender<(PingResult, Vec<PlayerInfo>)>,
}

impl Io for NetworkScanner {
    async fn ping(&self, addr: Ipv4Addr, port: u16) -> Result<(), eyre::Report> {
        let socket = TcpStream::connect(SocketAddr::new(addr.into(), port)).await?;
        socket.set_nodelay(true)?;
        let (socket_r, mut socket_w) = socket.into_split();

        let handshake_packet = ClientIntentionPacket {
            protocol_version: -1,
            hostname: String::from("snowstorm"),
            port: 42069,
            intention: ConnectionProtocol::Status,
        }
        .get();
        write_packet(&handshake_packet, &mut socket_w, None, &mut None).await?;

        let ping_packet = ServerboundStatusRequestPacket {}.get();
        write_packet(&ping_packet, &mut socket_w, None, &mut None).await?;

        let mut raw_read_connection = RawReadConnection {
            read_stream: socket_r,
            buffer: BytesMut::new(),
            compression_threshold: None,
            dec_cipher: None,
        };

        let clientbound_status = deserialize_packet::<ClientboundStatusPacket>(&mut Cursor::new(
            raw_read_connection.read().await?.as_slice(),
        ));

        if let Ok(ClientboundStatusPacket::StatusResponse(ping_response)) = clientbound_status {
            let ping_result = PingResult::from_azalea(addr, port, &ping_response);
            let player_info = PlayerInfo::from_azalea(&ping_response);
            self.sender.send((ping_result, player_info))?;
        };

        Ok(())
    }

    async fn legacy_ping(&self, _addr: Ipv4Addr, _port: u16) -> Result<(), eyre::Report> {
        todo!()
    }
}

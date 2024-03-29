use super::Io;
use crate::ScannerState;
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
use database::{player::PlayerInfo, server::PingResult};
use std::{
    io::Cursor,
    net::SocketAddrV4,
    sync::{mpsc::Sender, Arc},
};
use tokio::{net::TcpStream, sync::Mutex};

pub struct NetworkScanner {
    pub state: Arc<Mutex<ScannerState>>,
    pub sender: Sender<(PingResult, Vec<PlayerInfo>)>,
}

impl Io for NetworkScanner {
    async fn ping(&mut self, addr: SocketAddrV4) -> Result<(), eyre::Report> {
        let socket = TcpStream::connect(addr).await?;
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
            let ping_result = PingResult::from_azalea(*addr.ip(), addr.port(), &ping_response);
            let player_info = PlayerInfo::from_azalea(&ping_response).await;
            self.sender.send((ping_result, player_info))?;
        };

        Ok(())
    }

    async fn legacy_ping(&mut self, _addr: SocketAddrV4) -> Result<(), eyre::Report> {
        todo!()
    }
}

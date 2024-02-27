use flate2::read::ZlibDecoder;
use std::{
    io::{Cursor, Read},
    net::SocketAddrV4,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use uuid::uuid;
use varint::{AsyncVarint, SyncVarintWrite};

pub mod varint;

#[derive(Debug, Default)]
pub struct JoinData {
    pub error: Option<String>,
}

impl JoinData {
    pub fn new() -> Self {
        Self::default()
    }
}

pub async fn join(addr: SocketAddrV4, version: i32) -> JoinData {
    let mut join_data = JoinData::new();

    let res = join_internal(addr, version, &mut join_data).await;
    if let Err(err) = res {
        join_data.error = Some(err.to_string());
    }

    join_data
}

async fn join_internal(
    addr: SocketAddrV4,
    version: i32,
    join_data: &mut JoinData,
) -> eyre::Result<()> {
    let mut stream = TcpStream::connect(addr).await?;

    let mut packet = Vec::new();
    packet.write_varint(0x00)?;
    packet.write_varint(version)?;
    let addr = b"shrecked.dev";
    packet.write_varint(addr.len() as i32)?;
    packet.write_all(addr).await?;
    packet.write_u16(42069).await?;
    packet.write_varint(2)?;
    write_packet(&mut stream, &packet, false).await?;

    let mut packet = Vec::new();
    packet.write_varint(0x00)?;
    let username = b"Test_bot";
    packet.write_varint(username.len() as i32)?;
    packet.write_all(username).await?;
    if version > 47 {
        // TODO: not sure when uuid field was added but its not in 1.8.x (47)
        let player_uuid = uuid!("36d4d63f-7268-4879-a57f-122e9df006c2").as_bytes();
        packet.write_all(player_uuid).await?;
    }
    write_packet(&mut stream, &packet, false).await?;

    let packet = read_packet(&mut stream, false).await?;
    println!("got packet = {packet:?}");
    match packet.0 {
        0 => {
            // kick
        }
        1 => {
            // encryption, online mode probably
        }
        3 => {
            // compression, offline mode probably
        }
        _ => {
            // unknown packet
        }
    };

    Ok(())
}

async fn write_packet(stream: &mut TcpStream, packet: &[u8], compressed: bool) -> eyre::Result<()> {
    let mut to_send = Vec::new();

    if compressed {
        todo!()
    } else {
        to_send.write_varint(packet.len() as i32)?;
        to_send.write_all(packet).await?;
    }

    println!("sending packet = {to_send:?}");

    stream.write_all(&to_send).await?;
    Ok(())
}

async fn read_packet(stream: &mut TcpStream, compressed: bool) -> eyre::Result<(i32, Vec<u8>)> {
    println!("a");
    let packet_length = stream.read_varint().await?;

    if compressed {
        let (data_length_size, data_length) = stream.read_varint_len().await?;
        if data_length != 0 {
            let mut buf = Vec::with_capacity(data_length as usize);
            stream.read_exact(&mut buf).await?;

            let mut z = ZlibDecoder::new(Cursor::new(buf));
            let mut buf = Vec::new();
            z.read_to_end(&mut buf)?;

            let mut buf = Cursor::new(buf);
            let (packet_id_size, packet_id) = buf.read_varint_len().await?;
            let mut packet = vec![0; packet_length as usize - packet_id_size as usize];
            stream.read_exact(&mut packet).await?;

            return Ok((packet_id, packet));
        }
    }

    let (packet_id_size, packet_id) = stream.read_varint_len().await?;
    let mut packet = vec![0; packet_length as usize - packet_id_size as usize];
    stream.read_exact(&mut packet).await?;

    Ok((packet_id, packet))
}

use flate2::read::ZlibDecoder;
use std::{
    io::{Cursor, Read},
    net::SocketAddrV4,
};
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::varint::AsyncVarint;

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

pub async fn join(addr: SocketAddrV4) -> JoinData {
    let mut join_data = JoinData::new();

    let res = join_internal(addr, &mut join_data).await;
    if let Err(err) = res {
        join_data.error = Some(err.to_string());
    }

    join_data
}

async fn join_internal(addr: SocketAddrV4, join_data: &mut JoinData) -> eyre::Result<()> {
    let mut stream = TcpStream::connect(addr).await?;

    let packet = read_packet(&mut stream, false).await?;
    println!("packet = {packet:?}");

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

use std::{net::SocketAddrV4, str::FromStr};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let data = bunger::join(SocketAddrV4::from_str("130.61.123.184:25565").unwrap()).await;
    println!("data = {data:?}");

    Ok(())
}

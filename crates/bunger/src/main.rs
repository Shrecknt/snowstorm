use std::{net::SocketAddrV4, str::FromStr};

use database::{server::PingResult, DatabaseConnection};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let addr = SocketAddrV4::from_str("127.0.0.1:25569").unwrap();

    let db = DatabaseConnection::new().await.unwrap();
    let server_id = PingResult::from_ip_port(addr.ip(), addr.port(), &db.pool)
        .await
        .map(|res| res.id.unwrap())
        .unwrap_or(0);

    let mut server =
        ram_server::run_server("1.8.9", 25569, false).expect("unable to start server :<");

    let data = bunger::join(addr, 47, server_id).await;

    server.kill().expect("Unable to kill child process");

    println!("data = {data:?}");

    Ok(())
}

use std::{net::SocketAddrV4, str::FromStr};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let mut server =
        ram_server::run_server("1.8.9", 25569, false).expect("unable to start server :<");

    let data = bunger::join(SocketAddrV4::from_str("127.0.0.1:25569").unwrap(), 47).await;

    server.kill().expect("Unable to kill child process");

    println!("data = {data:?}");

    Ok(())
}

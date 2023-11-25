use super::PORTS_RANGE;
use crate::io::Io;
use std::net::Ipv4Addr;

pub async fn all_ports<T: Io>(
    pinger: &T,
    cursor: &mut u32,
    ip: Ipv4Addr,
) -> Result<(), eyre::Report> {
    for port in PORTS_RANGE {
        pinger.ping(ip, port).await?;
    }
    Ok(())
}

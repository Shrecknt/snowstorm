use super::PORTS_RANGE;
use crate::io::Io;
use std::net::Ipv4Addr;

pub async fn all_ports<T: Io>(pinger: &T, cursor: &mut u32, ip: Ipv4Addr) -> eyre::Result<()> {
    if !PORTS_RANGE.contains(&(*cursor as u16)) {
        *cursor = PORTS_RANGE.start as u32;
        return Ok(());
    }
    pinger.ping(ip, *cursor as u16).await?;
    *cursor += 1;
    Ok(())
}

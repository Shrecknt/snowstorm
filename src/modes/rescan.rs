use crate::{io::Io, util::exclude};
use std::net::Ipv4Addr;

pub async fn rescan<T: Io>(
    pinger: &mut T,
    cursor: &mut usize,
    ips: &[(Ipv4Addr, u16)],
) -> eyre::Result<()> {
    if *cursor >= ips.len() {
        *cursor = 0;
    }
    let addr = match ips.get(*cursor) {
        Some(addr) => addr,
        None => return Ok(()),
    };
    *cursor += 1;
    if exclude::is_allowed(addr.0) {
        pinger.ping(addr.0, addr.1).await?;
    }
    Ok(())
}

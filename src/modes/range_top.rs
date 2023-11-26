use super::TOP_PORTS;
use crate::{addr_range::Ipv4AddrRange, exclude, io::Io};
use std::net::Ipv4Addr;

#[allow(unused)]
pub async fn range_top<T: Io>(
    pinger: &T,
    cursor: &mut u32,
    range: Ipv4AddrRange,
) -> eyre::Result<()> {
    let mut ip = Ipv4Addr::from(*cursor);

    if !range.contains(ip) {
        *cursor = u32::from(range.first);
        return Ok(());
    }
    if !exclude::is_allowed(ip) {
        *cursor += 1;
        return Ok(());
    }

    for port in TOP_PORTS {
        pinger.ping(ip, port).await?;
    }
    *cursor += 1;

    Ok(())
}

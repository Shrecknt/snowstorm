use std::net::Ipv4Addr;

use crate::{
    io::Io,
    util::{addr_range::Ipv4AddrRange, exclude},
};

#[allow(unused)]
pub async fn range<T: Io>(pinger: &T, cursor: &mut u32, range: &Ipv4AddrRange) -> eyre::Result<()> {
    let mut ip = Ipv4Addr::from(*cursor);

    if !range.contains(ip) {
        *cursor = u32::from(range.first);
        return Ok(());
    }
    if !exclude::is_allowed(ip) {
        *cursor += 1;
        return Ok(());
    }

    pinger.ping(ip, 25565).await?;
    *cursor += 1;

    Ok(())
}

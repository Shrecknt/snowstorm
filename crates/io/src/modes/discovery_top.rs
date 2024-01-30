use super::{RANDOMIZER, TOP_PORTS};
use crate::Io;
use common::exclude;
use std::net::Ipv4Addr;

pub async fn discovery_top<T: Io>(pinger: &mut T, cursor: &mut u32) -> eyre::Result<()> {
    let mut ip: Ipv4Addr;
    loop {
        ip = Ipv4Addr::from(RANDOMIZER.shuffle(*cursor as u64) as u32);
        *cursor += 1;
        if exclude::is_allowed(ip) {
            break;
        }
    }
    for port in TOP_PORTS {
        pinger.ping(ip, port).await?;
    }
    Ok(())
}

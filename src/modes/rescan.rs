use crate::{exclude, io::Io};
use std::net::Ipv4Addr;

pub async fn rescan<T: Io>(
    pinger: &T,
    cursor: &mut usize,
    ips: &Vec<(Ipv4Addr, u16)>,
) -> eyre::Result<()> {
    if *cursor >= ips.len() {
        *cursor = 0;
        // I know this looks dumb, but if
        // there was no return, getting the
        // addr from the cursor would panic
        return Ok(());
    }
    let addr = ips[*cursor];
    *cursor += 1;
    if exclude::is_allowed(addr.0) {
        pinger.ping(addr.0, addr.1).await?;
    }
    Ok(())
}

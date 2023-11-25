use crate::{addr_range::Ipv4AddrRange, io::Io};

#[allow(unused)]
pub async fn range_top<T: Io>(
    pinger: &T,
    cursor: &mut u32,
    range: Ipv4AddrRange,
) -> eyre::Result<()> {
    todo!()
}

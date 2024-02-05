use common::addr_range::Ipv4AddrRange;
use std::{collections::HashSet, net::Ipv4Addr};

pub fn get_slash24(ip: Ipv4Addr) -> Ipv4AddrRange {
    let bits = u32::from(ip);
    Ipv4AddrRange::new(
        Ipv4Addr::from(bits & 0xffffff00),
        Ipv4Addr::from(bits & 0xffffffff),
    )
}

pub fn get_slash24s(ips: &Vec<Ipv4Addr>) -> Vec<Ipv4AddrRange> {
    let mut ranges = HashSet::new();
    for ip in ips {
        let range = get_slash24(*ip);
        if !ranges.contains(&range) {
            ranges.insert(range);
        }
    }
    ranges.iter().copied().collect()
}

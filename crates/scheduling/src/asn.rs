use common::addr_range::Ipv4AddrRange;
use dashmap::{DashMap, DashSet};
use rayon::prelude::*;
use std::net::Ipv4Addr;

pub fn get_slash24(ip: Ipv4Addr) -> Ipv4AddrRange {
    let bits = u32::from(ip);
    Ipv4AddrRange::new(
        Ipv4Addr::from(bits & 0xffffff00),
        Ipv4Addr::from(bits | 0x000000ff),
    )
}

pub async fn get_slash24s(ips: &Vec<Ipv4Addr>) -> Vec<Ipv4AddrRange> {
    tokio::task::yield_now().await;
    let ranges = DashSet::new();
    for ip in ips {
        let range = get_slash24(*ip);
        if !ranges.contains(&range) {
            ranges.insert(range);
        }
    }
    ranges.par_iter().map(|v| *v).collect()
}

pub async fn get_slash24s_map_key(ips: &DashMap<Ipv4Addr, usize>) -> DashMap<Ipv4AddrRange, usize> {
    tokio::task::yield_now().await;
    let keys = ips.iter().map(|ip| get_slash24(*ip.key()));
    let res = DashMap::new();
    for (k, v) in keys.zip(ips.iter().map(|v| *v.value())) {
        if let Some(mut value) = res.get_mut(&k) {
            *value += v;
        } else {
            res.insert(k, v);
        }
    }
    res
}

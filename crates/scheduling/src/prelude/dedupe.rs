use super::Dedupe;
use common::{addr_range::Ipv4AddrRange, network_range::SocketAddrV4Range};
use dashmap::DashMap;
use std::net::{Ipv4Addr, SocketAddrV4};

impl Dedupe for DashMap<Ipv4AddrRange, usize> {
    fn dedupe(&self) -> Self {
        let res = DashMap::new();
        for pair in self.iter_mut() {
            let k = pair.key();
            let v = pair.value();
            if let Some(mut value) = res.get_mut(k) {
                *value += v;
            } else {
                res.insert(*k, *v);
            }
        }
        res
    }
}

impl Dedupe for DashMap<SocketAddrV4Range, usize> {
    fn dedupe(&self) -> Self {
        todo!()
    }
}

impl Dedupe for Vec<Ipv4Addr> {
    fn dedupe(&self) -> Self {
        todo!()
    }
}

impl Dedupe for Vec<SocketAddrV4> {
    fn dedupe(&self) -> Self {
        todo!()
    }
}

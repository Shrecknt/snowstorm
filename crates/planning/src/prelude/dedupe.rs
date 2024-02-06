use super::Dedupe;
use common::{addr_range::Ipv4AddrRange, network_range::SocketAddrV4Range};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
};

impl Dedupe for HashMap<Ipv4AddrRange, usize> {
    fn dedupe(&self) -> Self {
        let mut res = HashMap::new();
        for (k, v) in self {
            if let Some(value) = res.get_mut(k) {
                *value += v;
            } else {
                res.insert(*k, *v);
            }
        }
        res
    }
}

impl Dedupe for HashMap<SocketAddrV4Range, usize> {
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

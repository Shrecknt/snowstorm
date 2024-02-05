use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SocketAddrV4Range {
    pub start: SocketAddrV4,
    pub end: SocketAddrV4,
    pub count: u64,
}

impl SocketAddrV4Range {
    pub fn new(start: SocketAddrV4, end: SocketAddrV4) -> Self {
        Self {
            start,
            end,
            count: 0,
        }
    }
    pub fn new_with_count(start: SocketAddrV4, end: SocketAddrV4, count: u64) -> Self {
        Self { start, end, count }
    }

    pub fn contains(&self, addr: &SocketAddrV4) -> bool {
        addr.ip() >= self.start.ip()
            && addr.ip() <= self.end.ip()
            && addr.port() >= self.start.port()
            && addr.port() <= self.end.port()
    }

    pub fn overlaps(&self, other: &SocketAddrV4Range) -> bool {
        self.contains(&other.start) || self.contains(&other.end) || other.contains(&self.start)
    }

    pub fn merge(&mut self, other: &SocketAddrV4Range) {
        if self.start.ip() > other.start.ip() {
            self.start.set_ip(*other.start.ip());
        }
        if self.end.ip() < other.end.ip() {
            self.end.set_ip(*other.end.ip());
        }
        if self.start.port() > other.start.port() {
            self.start.set_port(other.start.port());
        }
        if self.end.port() < other.end.port() {
            self.end.set_port(other.end.port());
        }
        self.count += other.count;
    }

    pub fn expand_to_asn(&mut self) {}

    pub fn expand_range(&mut self, amount: u32) {
        self.start
            .set_ip((u32::from(*self.start.ip()).saturating_sub(amount)).into());
        self.end
            .set_ip((u32::from(*self.end.ip()).saturating_add(amount)).into());
    }

    pub fn expand_port(&mut self, amount: u16) {
        self.start
            .set_port(self.start.port().saturating_sub(amount));
        self.end.set_port(self.end.port().saturating_add(amount));
    }

    pub fn remove_overlap(mut ranges: Vec<Self>) {
        if ranges.is_empty() {
            return;
        }
        let mut to_remove = Vec::new();
        let mut current = 0;
        for index in 1..ranges.len() {
            let new = &ranges[index];
            if ranges[current].overlaps(new) {
                let new = new.clone();
                ranges[current].merge(&new);
                to_remove.push(index);
            } else {
                current = index;
            }
        }
        for index in to_remove.iter().rev() {
            ranges.remove(*index);
        }
    }
}

impl PartialOrd for SocketAddrV4Range {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start.partial_cmp(&other.start) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.end.partial_cmp(&other.end)
    }
}

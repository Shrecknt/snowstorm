use crate::addr_range::Ipv4AddrRange;
use lazy_static::lazy_static;
use std::{cmp::Ordering, collections::BTreeSet, fs, net::Ipv4Addr};

lazy_static! {
    static ref EXCLUDE_LIST: BTreeSet<ExcludeEntry> = {
        let file = fs::read_to_string("exclude.txt").unwrap();
        let rows = file.split('\n').filter(|item| item != &"");
        rows.map(|ip_string| {
            let ip_string = ip_string.trim();
            if ip_string.contains('/') {
                let range = ip_string.parse::<Ipv4AddrRange>().unwrap();
                ExcludeEntry::Range(range)
            } else if ip_string.contains('-') {
                let sections = ip_string.split('-').collect::<Vec<_>>();
                let range_start = sections[0].parse().unwrap();
                let range_end = sections[1].parse().unwrap();
                ExcludeEntry::Range(Ipv4AddrRange::new(range_start, range_end))
            } else {
                ExcludeEntry::Address(
                    ip_string
                        .parse()
                        .unwrap_or_else(|_| panic!("Invalid ip string '{}'", ip_string)),
                )
            }
        })
        .collect()
    };
}

#[derive(Debug, Clone)]
pub enum ExcludeEntry {
    Address(Ipv4Addr),
    Range(Ipv4AddrRange),
}

impl PartialEq for ExcludeEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for ExcludeEntry {}

impl PartialOrd for ExcludeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExcludeEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            ExcludeEntry::Address(a) => match other {
                ExcludeEntry::Address(b) => a.cmp(b),
                ExcludeEntry::Range(b) => {
                    if a < &b.first {
                        Ordering::Less
                    } else if a > &b.last {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                }
            },
            ExcludeEntry::Range(a) => match other {
                ExcludeEntry::Address(b) => {
                    if &a.last < b {
                        Ordering::Less
                    } else if &a.first > b {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                }
                ExcludeEntry::Range(b) => a.first.cmp(&b.first),
            },
        }
    }
}

pub fn is_allowed(ip: Ipv4Addr) -> bool {
    !EXCLUDE_LIST.contains(&ExcludeEntry::Address(ip))
}

pub fn blocked_range_for_ip(ip: Ipv4Addr) -> Option<ExcludeEntry> {
    EXCLUDE_LIST.get(&ExcludeEntry::Address(ip)).cloned()
}

pub fn next_allowed(ip: Ipv4Addr) -> Ipv4Addr {
    let initial_ip = ip;
    let mut ip = ip;
    while !is_allowed(ip) {
        let exclude_entry = blocked_range_for_ip(ip).unwrap();
        match exclude_entry {
            ExcludeEntry::Address(_) => ip = Ipv4Addr::from(u32::from(ip) + 1),
            ExcludeEntry::Range(range) => ip = Ipv4Addr::from(u32::from(range.last) + 1),
        }
        if initial_ip == ip {
            panic!("someone excluded every fucking ip in existence");
        }
    }
    ip
}

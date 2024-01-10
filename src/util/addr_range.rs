use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, str::FromStr};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Ipv4AddrRange {
    pub first: Ipv4Addr,
    pub last: Ipv4Addr,
}

impl Ipv4AddrRange {
    pub fn new(first: Ipv4Addr, last: Ipv4Addr) -> Self {
        Self { first, last }
    }

    pub fn contains(&self, ip: Ipv4Addr) -> bool {
        ip >= self.first && ip <= self.last
    }
}

impl FromStr for Ipv4AddrRange {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let string = s.split('/').collect::<Vec<_>>();
        if string.len() != 2 {
            return Err(eyre::Report::msg("Range string must have 1 '/' character"));
        }
        let ip_u32 = u32::from(Ipv4Addr::from_str(string[0])?);
        let mask = 32 - string[1].parse::<u8>()?;
        let mask_bits = 2u32.pow(mask as u32) - 1;

        let range_start = Ipv4Addr::from(ip_u32 & !mask_bits);
        let range_end = Ipv4Addr::from(ip_u32 | mask_bits);

        Ok(Self {
            first: range_start,
            last: range_end,
        })
    }
}

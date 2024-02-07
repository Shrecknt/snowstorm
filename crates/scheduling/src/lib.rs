#![feature(map_many_mut)]

use asn::{get_slash24, get_slash24s_map_key};
use common::network_range::SocketAddrV4Range;
use prelude::*;
use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};
use sqlx::PgPool;
use std::{
    collections::HashMap,
    hash::Hash,
    net::{Ipv4Addr, SocketAddrV4},
};

mod db;

pub mod asn;
pub mod constants;
pub mod prelude;

pub struct ModePicker {
    pub modes: HashMap<ScanningMode, usize>,
}

impl ModePicker {
    pub fn new() -> Self {
        let mut modes = HashMap::new();
        for variant in ScanningMode::iter() {
            modes.insert(variant, (usize::MAX as f64).powf(0.5) as usize);
        }
        Self { modes }
    }
    pub fn pick_random(&mut self) -> ScanningMode {
        if self.modes.values().sum::<usize>() == 0 {
            for count in self.modes.values_mut() {
                *count = (usize::MAX as f64).powf(0.5) as usize;
            }
        }
        let mut rng = rand::thread_rng();
        let random_weights = self.modes.values().map(|value| {
            let value = *value as i64;
            let mut new_value = value.saturating_add(rng.gen_range(-100..100));
            if new_value < 1 {
                new_value = 1;
            }
            new_value as usize
        });
        let weighted = WeightedIndex::new(random_weights).unwrap();
        *self.modes.keys().nth(weighted.sample(&mut rng)).unwrap()
    }
}

impl Default for ModePicker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, enum_utils::IterVariants)]
pub enum ScanningMode {
    /// /0 on 25565
    OnePortAllAddress,
    /// /0 on random top 20 port
    OneRandomPortAllAddress,
    /// /32 on 1024-65535
    AllPortSingleAddress,
    /// /24 on 1024-65535
    AllPortSingleRange,
}

impl ScanningMode {
    pub fn variants() -> Vec<Self> {
        Self::iter().collect()
    }
    pub fn is_enabled(&self) -> bool {
        match self {
            _ => true,
        }
    }
}

#[derive(PartialEq, Eq, Hash, sqlx::FromRow)]
struct PortWrapper {
    port: i16,
}
impl PortWrapper {
    pub fn port(&self) -> u16 {
        self.port as u16
    }
}

#[derive(PartialEq, Eq, Hash, sqlx::FromRow)]
struct IpWrapper {
    ip: i32,
}
impl IpWrapper {
    pub fn ip(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.ip as u32)
    }
}

impl ScanningMode {
    pub async fn get_addresses(
        &self,
        pool: &PgPool,
    ) -> Result<Vec<SocketAddrV4Range>, sqlx::Error> {
        match self {
            ScanningMode::OnePortAllAddress => {
                let ips = db::get_ips(pool).await?;
                let ip_ranges =
                    get_slash24s_map_key(&ips).select_many_random_weighted(u16::MAX as usize);
                let socket_addr_ranges = ip_ranges
                    .iter()
                    .map(|slash24| (*slash24, 25565).into())
                    .collect();
                Ok(socket_addr_ranges)
            }
            ScanningMode::OneRandomPortAllAddress => {
                let port = db::get_ports(pool)
                    .await?
                    .top(20)
                    .select_one_random_weighted();
                let ips = db::get_ips(pool).await?;
                let ip_ranges =
                    get_slash24s_map_key(&ips).select_many_random_weighted(u16::MAX as usize);
                let socket_addr_ranges = ip_ranges
                    .iter()
                    .map(|slash24| (*slash24, port).into())
                    .collect();
                Ok(socket_addr_ranges)
            }
            ScanningMode::AllPortSingleAddress => {
                let ips = db::get_ips(pool).await?;
                let ip = ips.select_one_random_weighted();
                Ok(vec![SocketAddrV4Range::new(
                    SocketAddrV4::new(ip, constants::MIN_PORT),
                    SocketAddrV4::new(ip, constants::MAX_PORT),
                )])
            }
            ScanningMode::AllPortSingleRange => {
                let ips = db::get_ips(pool).await?;
                let range = get_slash24(ips.select_one_random_weighted());
                Ok(vec![
                    (range, constants::MIN_PORT, constants::MAX_PORT).into()
                ])
            }
        }
    }
}

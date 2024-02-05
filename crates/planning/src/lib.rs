#![feature(map_many_mut)]

use common::network_range::SocketAddrV4Range;
use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};
use sqlx::PgPool;
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
};

mod asn;
mod db;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, enum_utils::IterVariants)]
pub enum ScanningMode {
    OnePortAllAddress,
    AllPortSingleAddress,
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
                let mut rng = rand::thread_rng();
                let ports = db::get_ports(pool).await?;
                let weighted = WeightedIndex::new(ports.values()).unwrap();
                let port = *ports.keys().nth(weighted.sample(&mut rng)).unwrap();
                let ips = db::get_ips(pool).await?;
                println!("count = {}", ips.keys().len());
                let weighted = WeightedIndex::new(ips.values()).unwrap();
                let mut keys = weighted.sample_iter(&mut rng);
                let ips = {
                    let mut res = Vec::new();
                    let ips = ips.keys().collect::<Vec<_>>();
                    for _ in 0..u16::MAX {
                        let ip = **ips.get(keys.next().unwrap()).unwrap();
                        res.push(ip);
                    }
                    res
                };
                let slash24s = asn::get_slash24s(&ips);
                let socket_addr_ranges = slash24s.iter().map(|slash24| {
                    SocketAddrV4Range::new(
                        SocketAddrV4::new(slash24.first, port),
                        SocketAddrV4::new(slash24.last, port),
                    )
                });
                Ok(socket_addr_ranges.collect())
            }
            ScanningMode::AllPortSingleAddress => todo!(),
        }
    }
}

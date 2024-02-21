#![feature(map_many_mut)]

use asn::{get_slash24, get_slash24s_map_key};
use common::network_range::SocketAddrV4Range;
use dashmap::DashMap;
use prelude::*;
use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};
use rayon::prelude::*;
use sqlx::PgPool;
use std::{
    hash::Hash,
    net::{Ipv4Addr, SocketAddrV4},
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};
use tokio::runtime::Runtime;

mod db;

pub mod asn;
pub mod constants;
pub mod prelude;

/// (usize::MAX as f64).powf(0.5) as u64
const DEFAULT_WEIGHT: u64 = 0x100000000;

#[derive(Debug)]
pub struct ModePicker {
    pub modes: DashMap<ScanningMode, u64>,
}

impl ModePicker {
    pub fn new() -> Self {
        let modes = DashMap::new();
        for variant in ScanningMode::iter() {
            if variant.is_enabled() {
                modes.insert(variant, 0);
            }
        }
        Self { modes }
    }
    pub fn new_all() -> Self {
        let modes = DashMap::new();
        for variant in ScanningMode::iter() {
            modes.insert(variant, 0);
        }
        Self { modes }
    }

    pub fn set(&mut self, variant: ScanningMode, count: u64) {
        if let Some(mut value) = self.modes.get_mut(&variant) {
            *value = count;
        } else {
            self.modes.insert(variant, count);
        }
    }

    pub fn update(&mut self, variant: ScanningMode, count: u64) {
        if let Some(mut value) = self.modes.get_mut(&variant) {
            if *value == 0 || *value == DEFAULT_WEIGHT {
                *value = count;
            } else {
                *value /= 4;
                *value += count * 3 / 4;
            }
        } else {
            self.modes.insert(variant, count);
        }
    }

    pub fn pick_random(&mut self) -> ScanningMode {
        if self.modes.is_empty() {
            panic!("ModePicker was incorrectly initialized, no values in self.modes\nTry calling ModePicker::new() or enabling some modes");
        }
        if self.modes.iter().all(|v| *v.value() == 0) {
            for mut count in self.modes.iter_mut() {
                *count.value_mut() = DEFAULT_WEIGHT;
            }
            return ScanningMode::OnePortAllAddress;
        }
        let mut rng = rand::thread_rng();
        let random_weights = self.modes.iter().map(|value| {
            let value = *value.value() as i64;
            let mut new_value = value.saturating_add(rng.gen_range(-100..=100));
            if new_value < 1 {
                new_value = 1;
            }
            new_value as usize
        });
        let weighted = WeightedIndex::new(random_weights).unwrap();
        *self
            .modes
            .iter()
            .nth(weighted.sample(&mut rng))
            .unwrap()
            .key()
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
    /// TODO: run on ranges even without Minecraft servers
    OnePortAllAddress,
    /// /0 on all top 10
    /// TODO: run on ranges even without Minecraft servers
    TopPortAllAddress,
    /// /24 with Minecraft servers on 25565
    OnePortMinecraftRange,
    /// /24 with Minecraft servers on top 100
    TopPortMinecraftRange,
    /// /24 with Minecraft servers on 1024-65535
    AllPortMinecraftRange,
    /// /32 with Minecraft server on 1024-65535
    AllPortSingleMinecraftAddress,
    /// /0 with Minecraft servers on random port
    OneRandomPortAllAddress,
    /// /32 with Minecraft servers on 1024-65535
    AllPortSingleMinecraftRange,
    /// /24 on 1024-65535
    AllPortSingleRange,
}

impl ScanningMode {
    pub fn variants() -> Vec<Self> {
        Self::iter().collect()
    }
    pub fn is_enabled(&self) -> bool {
        !matches!(
            self,
            Self::OneRandomPortAllAddress | Self::AllPortMinecraftRange | Self::AllPortSingleRange
        )
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
                let ip_ranges = get_slash24s_map_key(&ips)
                    .await
                    .select_many_random_weighted(u16::MAX as usize)
                    .await;
                let socket_addr_ranges = ip_ranges
                    .par_iter()
                    .map(|slash24| (*slash24, 25565).into())
                    .collect();
                Ok(socket_addr_ranges)
            }
            ScanningMode::TopPortAllAddress => {
                let ports = db::get_ports(pool).await?.top(10);
                let ports = ports.par_iter().map(|v| *v.key()).collect::<Vec<_>>();
                let ips = db::get_ips(pool).await?;
                let ip_ranges = get_slash24s_map_key(&ips)
                    .await
                    .select_many_random_weighted(u16::MAX as usize)
                    .await;
                let mut socket_addr_ranges = Vec::new();
                for slash24 in ip_ranges {
                    for port in &ports {
                        socket_addr_ranges.push((slash24, *port).into());
                    }
                }
                Ok(socket_addr_ranges)
            }
            ScanningMode::OnePortMinecraftRange => {
                let ips = db::get_ips(pool).await?;
                let ip_ranges = get_slash24s_map_key(&ips)
                    .await
                    .select_many_random_weighted(u16::MAX as usize)
                    .await;
                let socket_addr_ranges = ip_ranges
                    .par_iter()
                    .map(|slash24| (*slash24, 25565).into())
                    .collect();
                Ok(socket_addr_ranges)
            }
            ScanningMode::TopPortMinecraftRange => {
                let ports = db::get_ports(pool).await?.top(100);
                let ports = ports.par_iter().map(|v| *v.key()).collect::<Vec<_>>();
                let ips = db::get_ips(pool).await?;
                let ip_ranges = get_slash24s_map_key(&ips)
                    .await
                    .select_many_random_weighted(u16::MAX as usize)
                    .await;
                let mut socket_addr_ranges = Vec::new();
                for slash24 in ip_ranges {
                    for port in &ports {
                        socket_addr_ranges.push((slash24, *port).into());
                    }
                }
                Ok(socket_addr_ranges)
            }
            ScanningMode::AllPortMinecraftRange => {
                let ips = db::get_ips(pool).await?;
                let ip_ranges = get_slash24s_map_key(&ips)
                    .await
                    .select_many_random_weighted(u16::MAX as usize)
                    .await;
                let mut socket_addr_ranges = Vec::new();
                for slash24 in ip_ranges {
                    socket_addr_ranges
                        .push((slash24, constants::MIN_PORT, constants::MAX_PORT).into());
                }
                Ok(socket_addr_ranges)
            }
            ScanningMode::OneRandomPortAllAddress => {
                let port = db::get_ports(pool)
                    .await?
                    .top(20)
                    .select_one_random_weighted()
                    .await;
                let ips = db::get_ips(pool).await?;
                let ip_ranges = get_slash24s_map_key(&ips)
                    .await
                    .select_many_random_weighted(u16::MAX as usize)
                    .await;
                let socket_addr_ranges = ip_ranges
                    .par_iter()
                    .map(|slash24| (*slash24, port).into())
                    .collect();
                Ok(socket_addr_ranges)
            }
            ScanningMode::AllPortSingleMinecraftRange => {
                let ips = db::get_ips(pool).await?;
                let ip_range = get_slash24s_map_key(&ips)
                    .await
                    .select_one_random_weighted()
                    .await;

                Ok(vec![
                    (ip_range, constants::MIN_PORT, constants::MAX_PORT).into()
                ])
            }
            ScanningMode::AllPortSingleMinecraftAddress => {
                let ips = db::get_ips(pool).await?;
                let ip = ips.select_one_random_weighted().await;
                Ok(vec![SocketAddrV4Range::new(
                    SocketAddrV4::new(ip, constants::MIN_PORT),
                    SocketAddrV4::new(ip, constants::MAX_PORT),
                )])
            }
            ScanningMode::AllPortSingleRange => {
                let ips = db::get_ips(pool).await?;
                let range = get_slash24(ips.select_one_random_weighted().await);
                Ok(vec![
                    (range, constants::MIN_PORT, constants::MAX_PORT).into()
                ])
            }
        }
    }
}

pub fn start_scheduler_queue(
    sender: Sender<(ScanningMode, Vec<SocketAddrV4Range>)>,
    receiver: Receiver<Option<(ScanningMode, u64)>>,
    modes: Arc<parking_lot::Mutex<ModePicker>>,
    pool: PgPool,
) {
    std::thread::spawn(move || {
        Runtime::new().unwrap().block_on(async move {
            while let Ok(last_scan_results) = receiver.recv() {
                let new_mode = {
                    let mut modes_lock = modes.lock();
                    if let Some((mode, count)) = last_scan_results {
                        modes_lock.update(mode, count);
                    }
                    modes_lock.pick_random()
                };
                let addresses = new_mode.get_addresses(&pool).await.unwrap();
                sender.send((new_mode, addresses)).unwrap();
            }
        });
    });
}

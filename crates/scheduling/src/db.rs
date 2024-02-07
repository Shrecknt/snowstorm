use crate::{IpWrapper, PortWrapper};
use dashmap::DashMap;
use sqlx::PgPool;
use std::net::Ipv4Addr;

pub async fn get_ports(pool: &PgPool) -> Result<DashMap<u16, usize>, sqlx::Error> {
    let map = DashMap::new();
    let ports: Vec<PortWrapper> = sqlx::query_as("SELECT port FROM servers LIMIT 10000000")
        .fetch_all(pool)
        .await?;
    for port in ports {
        let port = port.port();
        if let Some(mut value) = map.get_mut(&port) {
            *value += 1;
        } else {
            map.insert(port, 1);
        }
    }
    Ok(map)
}

pub async fn get_ips(pool: &PgPool) -> Result<DashMap<Ipv4Addr, usize>, sqlx::Error> {
    let map = DashMap::new();
    let ips: Vec<IpWrapper> = sqlx::query_as("SELECT ip FROM servers LIMIT 10000000")
        .fetch_all(pool)
        .await?;
    for ip in ips {
        let ip = ip.ip();
        if let Some(mut value) = map.get_mut(&ip) {
            *value += 1;
        } else {
            map.insert(ip, 1);
        }
    }
    Ok(map)
}

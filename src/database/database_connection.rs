use dotenvy_macro::dotenv as var;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::net::Ipv4Addr;

#[derive(Clone)]
pub struct DatabaseConnection {
    pub pool: PgPool,
}

impl DatabaseConnection {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(var!("DATABASE_URL"))
            .await?;
        Ok(Self { pool })
    }

    pub async fn get_rescan(&self) -> eyre::Result<Vec<(Ipv4Addr, u16)>> {
        let res = sqlx::query("SELECT id FROM servers")
            .fetch_all(&self.pool)
            .await?;
        Ok(res.iter().map(|id| from_id(id.get("id"))).collect())
    }
}

pub fn to_id(ip: Ipv4Addr, port: u16) -> i64 {
    (u32::from(ip) as i64) << 16 | (port as i64)
}

pub fn from_id(id: i64) -> (Ipv4Addr, u16) {
    (Ipv4Addr::from((id >> 16) as u32), (id & 0xFFFF) as u16)
}

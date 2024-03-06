use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Clone, Debug)]
pub struct DatabaseConnection {
    pub pool: PgPool,
}

impl DatabaseConnection {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(&config::get().database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn get_rescan(&self) -> eyre::Result<Vec<SocketAddrV4>> {
        let res = sqlx::query("SELECT ip, port FROM servers")
            .fetch_all(&self.pool)
            .await?;
        Ok(res
            .iter()
            .map(|id| {
                SocketAddrV4::new(
                    Ipv4Addr::from(id.get::<i32, _>("ip") as u32),
                    id.get::<i16, _>("port") as u16,
                )
            })
            .collect())
    }
}

pub use database_connection::DatabaseConnection;

use self::{player::PlayerInfo, server::PingResult};

pub mod autocomplete;
pub mod database_connection;
pub mod discord_user;
pub mod forgejo_user;
pub mod player;
pub mod server;
pub mod server_joins;
pub mod user;

pub trait DbPush {
    fn push(
        &mut self,
        pool: &sqlx::PgPool,
    ) -> impl std::future::Future<Output = Result<(), eyre::Report>> + Send;
}

impl DbPush for (PingResult, Vec<PlayerInfo>) {
    async fn push(&mut self, pool: &sqlx::PgPool) -> Result<(), eyre::Report> {
        self.0.push(pool).await?;
        let id = self.0.id.unwrap();
        for player in &mut self.1 {
            player.push(id, pool).await?;
        }
        Ok(())
    }
}

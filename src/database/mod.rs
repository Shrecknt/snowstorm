pub use database_connection::DatabaseConnection;

pub mod database_connection;
pub mod discord_user;
pub mod user;
pub mod server;
pub mod player;

pub trait DbPush {
    fn push(
        &mut self,
        pool: &sqlx::PgPool,
    ) -> impl std::future::Future<Output = Result<(), eyre::Report>> + Send;
}

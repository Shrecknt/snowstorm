use database::DatabaseConnection;
use discord::run_bot;

#[tokio::main]
async fn main() {
    let db = DatabaseConnection::new().await.unwrap();
    run_bot(&db.pool).await;
}

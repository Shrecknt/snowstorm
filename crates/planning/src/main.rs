use std::time::Instant;

use database::DatabaseConnection;
use planning::ScanningMode;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().expect(".env file not found");

    let db = DatabaseConnection::new().await?;

    let mode = ScanningMode::OnePortAllAddress;

    let start_time = Instant::now();
    let ranges = mode.get_addresses(&db.pool).await?;
    let end_time = Instant::now();

    let run_time = end_time - start_time;

    // println!("ranges = {ranges:?}");
    println!("ranges len = {:?}", ranges.len());
    println!("time to run {run_time:?}");

    Ok(())
}

use database::DatabaseConnection;
use scheduling::ScanningMode;
use std::io::Write;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().expect(".env file not found");

    let db = DatabaseConnection::new().await?;

    for mode in ScanningMode::variants() {
        print!("Testing {:?}...", mode);
        std::io::stdout().flush().unwrap();
        let start_time = std::time::Instant::now();

        let ranges = mode.get_addresses(&db.pool).await?;

        let end_time = std::time::Instant::now();
        let run_time = end_time - start_time;
        println!(" took {:?} ({} range(s))", run_time, ranges.len());
    }

    Ok(())
}

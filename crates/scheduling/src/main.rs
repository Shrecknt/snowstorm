use database::DatabaseConnection;
use scheduling::{ModePicker, ScanningMode};
use std::io::Write;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().expect(".env file not found");

    let db = DatabaseConnection::new().await?;

    let mut modes = ModePicker::new_all();
    for _ in 0..ScanningMode::variants().len() * 2 {
        let mode = modes.pick_random();
        print!("Running {:?}...", mode);
        std::io::stdout().flush().unwrap();
        let ranges = mode.get_addresses(&db.pool).await?;
        modes.update(mode, ranges.len());
        println!(
            " found {} ranges, new state: {:?}",
            ranges.len(),
            modes.modes.iter().map(|v| *v.value()).collect::<Vec<_>>()
        );
    }

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

use database::DatabaseConnection;
use planning::ScanningMode;

macro_rules! test_modes {
    ($($mode:expr),*) => {
        {
            $(
                let db = DatabaseConnection::new().await?;
                println!("Testing {:?}", $mode);
                let start_time = std::time::Instant::now();

                let _ranges = $mode.get_addresses(&db.pool).await?;

                let end_time = std::time::Instant::now();
                let run_time = end_time - start_time;
                println!("Finished testing {:?}, took {:?}", $mode, run_time);
            )*
        }
    };
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().expect(".env file not found");

    test_modes!(
        ScanningMode::OnePortAllAddress,
        ScanningMode::AllPortSingleAddress,
        ScanningMode::AllPortSingleRange
    );

    Ok(())
}

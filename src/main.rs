use snowstorm::{
    database::DatabaseConnection,
    io::{database::DatabaseScanner, Io},
    modes::{self, ModeCursors, ScanningMode},
    ScannerState,
};
use std::{
    sync::{mpsc::channel, Arc},
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv::dotenv()?;

    let db = DatabaseConnection::new().await?;
    let state = Arc::new(Mutex::new(ScannerState::default()));
    let (ping_results_sender, ping_results) = channel();
    let pinger = DatabaseScanner::new(state.clone(), ping_results_sender);

    // println!("data: {:?}", pinger.data);

    let rescan = db.get_rescan().await?;
    println!("rows = {:?}", rescan);

    tokio::spawn(async move {
        ping_loop(db, state.clone(), pinger).await.unwrap();
    });

    for (result, _players) in ping_results {
        println!("{}:{}", result.ip, result.port);
    }

    Ok(())
}

async fn ping_loop<T: Io>(
    db: DatabaseConnection,
    state: Arc<Mutex<ScannerState>>,
    pinger: T,
) -> eyre::Result<()> {
    let mut cursors = ModeCursors::new();

    let mut time = Instant::now();
    loop {
        let mut state = state.lock().await;

        let now = Instant::now();
        if now - time > Duration::from_millis(1000) {
            time = now;
            println!(
                "Servers found for ScanningMode::{:?}: {}",
                state.mode, state.discovered
            );
            state.discovered = 0;
            if let ScanningMode::Rescan(..) = state.mode {
                // TODO decide on the best scanning mode to use
                state.mode = ScanningMode::Discovery;
            } else {
                let rescan_data = db.get_rescan().await.unwrap();
                state.mode = ScanningMode::Rescan(rescan_data);
            }
        }

        let mode = state.mode.clone();
        drop(state); // prevent deadlock

        match mode {
            ScanningMode::Discovery => {
                modes::discovery(&pinger, &mut cursors.discovery).await?;
            }
            ScanningMode::DiscoveryTopPorts => {
                modes::discovery_top(&pinger, &mut cursors.discovery_top_ports).await?;
            }
            ScanningMode::Range(range) => {
                modes::range(&pinger, &mut cursors.range, range).await?;
            }
            ScanningMode::RangeTopPorts(range) => {
                modes::range_top(&pinger, &mut cursors.range_top_ports, range).await?;
            }
            ScanningMode::AllPorts(ip) => {
                modes::all_ports(&pinger, &mut cursors.all_ports, ip).await?;
            }
            ScanningMode::Rescan(ips) => {
                modes::rescan(&pinger, &mut cursors.rescan, ips).await?;
            }
        }
    }
}

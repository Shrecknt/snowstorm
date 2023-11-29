use snowstorm::{
    database::DatabaseConnection,
    io::{database::DatabaseScanner, Io},
    modes::{self, ModeCursors, ScanningMode},
    web, ScannerState,
};
use std::{
    collections::LinkedList,
    sync::{mpsc::channel, Arc},
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv::dotenv()?;

    let db = Arc::new(Mutex::new(DatabaseConnection::new().await?));
    let state = Arc::new(Mutex::new(ScannerState::default()));
    let (ping_results_sender, ping_results) = channel();
    let task_queue = Arc::new(Mutex::new(LinkedList::new()));
    let pinger = DatabaseScanner::new(state.clone(), ping_results_sender);

    // println!("data: {:?}", pinger.data);

    let rescan = db.lock().await.get_rescan().await?;
    println!("rows = {:?}", rescan);

    {
        let db = db.clone();
        let state = state.clone();
        let task_queue = task_queue.clone();
        tokio::spawn(async move {
            ping_loop(db, state, task_queue, pinger).await.unwrap();
        });
    }

    {
        let db = db.clone();
        let state = state.clone();
        let task_queue = task_queue.clone();
        tokio::spawn(async move {
            web::start_server(db, state, task_queue).await.unwrap();
        });
    }

    for (result, players) in ping_results {
        println!(
            "{{ ip = '{}:{}', players = '{:?}' }}",
            result.ip, result.port, players
        );
    }

    Ok(())
}

async fn ping_loop<T: Io>(
    db: Arc<Mutex<DatabaseConnection>>,
    state: Arc<Mutex<ScannerState>>,
    task_queue: Arc<Mutex<LinkedList<(ScanningMode, Duration)>>>,
    pinger: T,
) -> eyre::Result<()> {
    let mut cursors = ModeCursors::new();
    let mut mode = ScanningMode::Paused;
    let mut current_mode_duration = Duration::MAX;

    let mut time = Instant::now();
    loop {
        let now = Instant::now();
        if now - time > current_mode_duration {
            time = now;

            let mut state = state.lock().await;

            println!(
                "Servers found for ScanningMode::{:?}: {}",
                mode, state.discovered
            );
            state.discovered = 0;

            drop(state); // prevent deadlock

            current_mode_duration = Duration::from_millis(1000);
            if let Some((new_mode, duration)) = task_queue.lock().await.pop_front() {
                mode = new_mode;
                current_mode_duration = duration;
            } else if let ScanningMode::Rescan(..) = mode {
                // TODO decide on the best scanning mode to use
                mode = ScanningMode::Discovery;
            } else {
                let rescan_data = db.lock().await.get_rescan().await.unwrap();
                mode = ScanningMode::Rescan(rescan_data);
            }
        }

        match mode {
            ScanningMode::Paused => {}
            ScanningMode::Discovery => {
                modes::discovery(&pinger, &mut cursors.discovery).await?;
            }
            ScanningMode::DiscoveryTopPorts => {
                modes::discovery_top(&pinger, &mut cursors.discovery_top_ports).await?;
            }
            ScanningMode::Range(ref range) => {
                modes::range(&pinger, &mut cursors.range, range).await?;
            }
            ScanningMode::RangeTopPorts(ref range) => {
                modes::range_top(&pinger, &mut cursors.range_top_ports, range).await?;
            }
            ScanningMode::AllPorts(ip) => {
                modes::all_ports(&pinger, &mut cursors.all_ports, ip).await?;
            }
            ScanningMode::Rescan(ref ips) => {
                modes::rescan(&pinger, &mut cursors.rescan, ips).await?;
            }
        }
    }
}

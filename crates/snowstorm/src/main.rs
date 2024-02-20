#![feature(linked_list_remove)]

use database::{player::PlayerInfo, server::PingResult, DatabaseConnection, DbPush};
use io::{
    modes::{self, ModeCursors, ScanningMode},
    Io, ScannerState,
};
use std::{
    collections::LinkedList,
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use tokio::{runtime::Runtime, sync::Mutex};
use web::actions::action::Action;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = config::get();
    let db = DatabaseConnection::new().await?;
    let state = Arc::new(Mutex::new(ScannerState::default()));
    let (ping_results_sender, ping_results) = channel();
    let mode_queue = Arc::new(Mutex::new(LinkedList::new()));
    let action_queue = Arc::new(Mutex::new(LinkedList::new()));

    #[cfg(debug_assertions)]
    let pinger = io::database::DatabaseScanner::new(state.clone(), ping_results_sender);
    #[cfg(not(debug_assertions))]
    let pinger = io::pnet::PnetScanner::new(state.clone(), ping_results_sender);

    if config.scanner.enabled {
        let db = db.clone();
        let state = state.clone();
        let mode_queue = mode_queue.clone();
        let action_queue = action_queue.clone();
        tokio::spawn(async move {
            ping_loop(db, state, mode_queue, action_queue, pinger)
                .await
                .unwrap();
        });
    }

    if config.web.enabled {
        let db = db.clone();
        let state = state.clone();
        let mode_queue = mode_queue.clone();
        let action_queue = action_queue.clone();
        tokio::spawn(async move {
            web::start_server(db, state, mode_queue, action_queue)
                .await
                .unwrap();
        });
    }

    if config.bot.enabled {
        let db = db.clone();
        tokio::spawn(async move {
            discord::run_bot(&db.pool).await;
        });
    }

    const CHANNEL_COUNT: usize = 8;

    let ping_handlers = {
        let mut handlers: Vec<Sender<_>> = Vec::with_capacity(CHANNEL_COUNT);
        for _ in 0..CHANNEL_COUNT {
            let db = db.clone();
            let (w, r) = channel::<(PingResult, Vec<PlayerInfo>)>();
            handlers.push(w);
            thread::spawn(move || {
                let r = r;
                while let Ok(mut values) = r.recv() {
                    Runtime::new()
                        .unwrap()
                        .block_on(values.push(&db.pool))
                        .unwrap();
                }
            });
        }
        handlers
    };

    {
        let mut handler_iter = 0;
        for result in ping_results {
            ping_handlers[handler_iter].send(result)?;
            handler_iter += 1;
            if handler_iter >= CHANNEL_COUNT {
                handler_iter = 0;
            }
        }
    }

    Ok(())
}

async fn ping_loop<T: Io>(
    db: DatabaseConnection,
    state: Arc<Mutex<ScannerState>>,
    mode_queue: Arc<Mutex<LinkedList<(ScanningMode, Duration)>>>,
    action_queue: Arc<Mutex<LinkedList<Action>>>,
    mut pinger: T,
) -> eyre::Result<()> {
    let mut cursors = ModeCursors::new();
    let mut mode = ScanningMode::Paused {};
    let mut current_mode_duration = Duration::MAX;

    let mut time = Instant::now();
    loop {
        let now = Instant::now();
        let delta = now - time;

        if delta > current_mode_duration {
            time = now;

            let mut state = state.lock().await;

            println!(
                "Servers found for ScanningMode::{:?}: {}",
                mode, state.discovered
            );
            state.discovered = 0;

            drop(state); // prevent deadlock

            if let Some((new_mode, duration)) = mode_queue.lock().await.pop_front() {
                mode = new_mode;
                current_mode_duration = duration;
            } else {
                match &mode {
                    ScanningMode::Auto {} => {}
                    ScanningMode::Paused {} | ScanningMode::Rescan { .. } => {
                        mode = ScanningMode::Paused {};
                        current_mode_duration = Duration::MAX;
                    }
                    _ => {
                        let rescan_data = db.get_rescan().await.unwrap();
                        mode = ScanningMode::Rescan { ips: rescan_data };
                    }
                }
            }
        }

        match mode {
            ScanningMode::Paused {} => {}
            ScanningMode::Discovery {} => {
                modes::discovery(&mut pinger, &mut cursors.discovery).await?;
            }
            ScanningMode::DiscoveryTopPorts {} => {
                modes::discovery_top(&mut pinger, &mut cursors.discovery_top_ports).await?;
            }
            ScanningMode::Range { ref range } => {
                modes::range(&mut pinger, &mut cursors.range, range).await?;
            }
            ScanningMode::RangeTopPorts { ref range } => {
                modes::range_top(&mut pinger, &mut cursors.range_top_ports, range).await?;
            }
            ScanningMode::AllPorts { ip } => {
                modes::all_ports(&mut pinger, &mut cursors.all_ports, ip).await?;
            }
            ScanningMode::Rescan { ref ips } => {
                modes::rescan(&mut pinger, &mut cursors.rescan, ips).await?;
            }
            ScanningMode::Auto {} => {
                modes::auto(&mut pinger, &mut cursors).await?;
            }
        }

        if delta > Duration::from_millis(1000) {
            if let Some(action) = action_queue.lock().await.pop_front() {
                match action {
                    Action::SetMode {
                        mode: new_mode,
                        duration,
                    } => {
                        mode = new_mode;
                        current_mode_duration = duration;
                    }
                    Action::Skip {} => {
                        current_mode_duration = Duration::ZERO;
                    }
                    Action::Clear {} => {
                        mode = ScanningMode::Paused {};
                        current_mode_duration = Duration::MAX;
                        mode_queue.lock().await.clear();
                    }
                    Action::Pause {} => {
                        let remaining = current_mode_duration - delta;
                        mode_queue.lock().await.push_front((mode, remaining));
                        mode = ScanningMode::Paused {};
                        current_mode_duration = Duration::MAX;
                    }
                    Action::Dequeue { index } => {
                        mode_queue.lock().await.remove(index);
                    }
                    Action::Enqueue { mode, duration } => {
                        mode_queue.lock().await.push_back((mode, duration));
                    }
                }
            }
        }
    }
}

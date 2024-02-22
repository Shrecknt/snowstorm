#![feature(linked_list_remove)]

use common::network_range::RangesExt;
use database::{player::PlayerInfo, server::PingResult, DatabaseConnection, DbPush};
use io::{Io, ScannerState};
use scheduling::ModePicker;
use std::{
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use tokio::{runtime::Runtime, sync::Mutex};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = config::get();
    let db = DatabaseConnection::new().await?;
    let state = Arc::new(Mutex::new(ScannerState::default()));
    let (ping_results_sender, ping_results) = channel();

    #[cfg(debug_assertions)]
    let pinger = io::database::DatabaseScanner::new(state.clone(), ping_results_sender);
    #[cfg(not(debug_assertions))]
    let pinger = io::pnet::PnetScanner::new(state.clone(), ping_results_sender);

    if config.scanner.enabled {
        let db = db.clone();
        let state = state.clone();
        tokio::spawn(async move {
            ping_loop(db, state, pinger).await.unwrap();
        });
    }

    if config.web.enabled {
        let db = db.clone();
        let state = state.clone();
        tokio::spawn(async move {
            web::start_server(db, state).await.unwrap();
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
            let config = config.clone();
            let db = db.clone();
            let (w, r) = channel::<(PingResult, Vec<PlayerInfo>)>();
            handlers.push(w);
            thread::spawn(move || {
                let r = r;
                while let Ok(mut values) = r.recv() {
                    if config.scanner.push_to_db {
                        Runtime::new()
                            .unwrap()
                            .block_on(values.push(&db.pool))
                            .unwrap();
                    }
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

async fn ping_loop(
    db: DatabaseConnection,
    state: Arc<Mutex<ScannerState>>,
    mut pinger: impl Io,
) -> eyre::Result<()> {
    let config = config::get();

    let mut request_state = RequestState::None;
    let mode_picker = Arc::new(parking_lot::Mutex::new(ModePicker::new()));
    let requester = channel();
    let receiver = channel();
    scheduling::start_scheduler_queue(receiver.0, requester.1, mode_picker, db.pool);

    // We don't have any data yet, so request the scheduler for addresses without providing any data
    requester.0.send(None)?;
    let (mut current_mode, mut current_addresses) = receiver.1.recv()?;
    println!("got new state {current_mode:?}");
    let mut total_addresses = current_addresses.count_addresses();
    assert_ne!(total_addresses, 0);
    println!("total addresses = {total_addresses}");
    let mut index = 0;

    let mut last_update = Instant::now();
    loop {
        if index % 2u64.pow(16) == 0 {
            match request_state {
                RequestState::None => {
                    let current_time = Instant::now();
                    let duration_since_last_update = current_time - last_update;
                    if duration_since_last_update
                        > Duration::from_secs(config.scanner.mode_duration)
                    {
                        let discovered = state.lock().await.discovered;
                        println!("discovered {discovered} servers");
                        requester.0.send(Some((current_mode, discovered)))?;
                        request_state = RequestState::Requested;
                        println!("requesting new state");
                        continue;
                    }
                }
                RequestState::Requested => {
                    if let Ok(new_state) = receiver.1.try_recv() {
                        (current_mode, current_addresses) = new_state;
                        total_addresses = current_addresses.count_addresses();
                        assert_ne!(total_addresses, 0);
                        println!("total addresses = {total_addresses}");
                        index = 0;
                        request_state = RequestState::None;
                        last_update = Instant::now();
                        state.lock().await.discovered = 0;
                        println!("got new state {current_mode:?}");
                        continue;
                    }
                }
            }
        }
        if index >= total_addresses {
            let duration_since_last_update = Instant::now() - last_update;
            let discovered = state.lock().await.discovered * config.scanner.mode_duration
                / duration_since_last_update.as_secs().max(1);
            println!("discovered {discovered} servers (extrapolated to duration)");
            requester.0.send(Some((current_mode, discovered)))?;
            println!("requesting new state (ended early)");
            (current_mode, current_addresses) = receiver.1.recv()?;
            total_addresses = current_addresses.count_addresses();
            assert_ne!(total_addresses, 0);
            println!("total addresses = {total_addresses}");
            index = 0;
            request_state = RequestState::None;
            last_update = Instant::now();
            state.lock().await.discovered = 0;
            println!("got new state {current_mode:?}");
            continue;
        }
        let current_addr = current_addresses.get_addr_at(index);
        pinger.ping(current_addr).await?;
        index += 1;
    }
}

#[derive(PartialEq)]
enum RequestState {
    None,
    Requested,
}

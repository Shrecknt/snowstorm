use crate::{database::DatabaseConnection, modes::ScanningMode, ScannerState};
use std::{collections::LinkedList, sync::Arc};
use tokio::sync::Mutex;

pub async fn start_server(
    _db: Arc<Mutex<DatabaseConnection>>,
    _state: Arc<Mutex<ScannerState>>,
    _task_queue: Arc<Mutex<LinkedList<ScanningMode>>>,
) -> eyre::Result<()> {
    println!("TODO: start webserver");
    Ok(())
}

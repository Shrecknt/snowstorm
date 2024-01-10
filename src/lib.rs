use modes::ScanningMode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod database;
pub mod io;
pub mod modes;
pub mod util;
pub mod web;

pub struct ScannerState {
    pub discovered: usize,
}

impl ScannerState {
    pub fn new() -> Self {
        ScannerState { discovered: 0 }
    }
}

impl Default for ScannerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    SetMode(ScanningMode, Duration),
    Skip,
    Clear,
    Pause,
    Dequeue(usize),
    Enqueue(ScanningMode, Duration),
}

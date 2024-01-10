use std::time::Duration;

use modes::ScanningMode;

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

pub enum Action {
    ScanningMode(ScanningMode, Duration),
    Skip,
    Pause,
}

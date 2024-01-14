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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    SetMode {
        mode: ScanningMode,
        duration: Duration,
    },
    Skip {},
    Clear {},
    Pause {},
    Dequeue {
        index: usize,
    },
    Enqueue {
        mode: ScanningMode,
        duration: Duration,
    },
}

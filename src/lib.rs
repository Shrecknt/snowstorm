use modes::ScanningMode;

pub mod addr_range;
pub mod database;
pub mod exclude;
pub mod io;
pub mod modes;
pub mod web;

pub struct ScannerState {
    pub mode: ScanningMode,
    pub discovered: usize,
}

impl ScannerState {
    pub fn new() -> Self {
        ScannerState {
            mode: ScanningMode::Discovery,
            discovered: 0,
        }
    }
}

impl Default for ScannerState {
    fn default() -> Self {
        Self::new()
    }
}

use io::modes::ScanningMode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
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

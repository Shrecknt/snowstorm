use crate::util::addr_range::Ipv4AddrRange;
use lazy_static::lazy_static;
use perfect_rand::PerfectRng;
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, ops::Range};

mod all_ports;
mod auto;
mod discovery;
mod discovery_top;
mod range;
mod range_top;
mod rescan;

pub use all_ports::all_ports;
pub use auto::auto;
pub use discovery::discovery;
pub use discovery_top::discovery_top;
pub use range::range;
pub use range_top::range_top;
pub use rescan::rescan;

pub const PORTS_RANGE: Range<u16> = 1000..65535;
pub const TOP_PORTS: [u16; 5] = [25565, 25566, 25567, 25570, 25575];

lazy_static! {
    pub static ref RANDOMIZER: PerfectRng = PerfectRng::new(2u64.pow(32), 0xda0d71bc391d3c92, 3);
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ScanningMode {
    /// Don't ping any servers - This can be enabled from the web ui
    Paused,
    /// Ping port 25565 on all allowed IPs
    Discovery,
    /// Ping top 8 common ports on all allowed IPs
    DiscoveryTopPorts,
    /// Ping port 25565 on all IPs in a given range
    Range(Ipv4AddrRange),
    /// Ping top 8 common ports on all IPs in a given range
    RangeTopPorts(Ipv4AddrRange),
    /// Ping all allowed ports on a single IP
    AllPorts(Ipv4Addr),
    /// Ping all servers in the database
    Rescan(Vec<(Ipv4Addr, u16)>),
    /// Automatically chose and switch between modes
    Auto,
}

#[derive(Debug)]
pub struct ModeCursors {
    pub discovery: u32,
    pub discovery_top_ports: u32,
    pub range: u32,
    pub range_top_ports: u32,
    pub all_ports: u32,
    pub rescan: usize,
}

impl ModeCursors {
    pub fn new() -> Self {
        Self {
            discovery: 0,
            discovery_top_ports: 0,
            range: 0,
            range_top_ports: 0,
            all_ports: 0,
            rescan: 0,
        }
    }
}

impl Default for ModeCursors {
    fn default() -> Self {
        Self::new()
    }
}

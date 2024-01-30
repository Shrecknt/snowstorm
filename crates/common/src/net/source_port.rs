use serde::Deserialize;

#[derive(Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum SourcePort {
    Number(u16),
    Range { min: u16, max: u16 },
}

impl SourcePort {
    /// Pick a source port based on the given seed.
    ///
    /// If the source port is a range, then the port is chosen uniformly from
    /// the range. Otherwise, the port is the given number.
    pub fn pick(&self, seed: u32) -> u16 {
        match self {
            SourcePort::Number(port) => *port,
            SourcePort::Range { min, max } => {
                let range = max - min;
                (seed % range as u32) as u16 + min
            }
        }
    }

    pub fn contains(&self, port: u16) -> bool {
        match self {
            SourcePort::Number(p) => *p == port,
            SourcePort::Range { min, max } => *min <= port && port <= *max,
        }
    }
}

impl Default for SourcePort {
    fn default() -> Self {
        SourcePort::Number(61000)
    }
}

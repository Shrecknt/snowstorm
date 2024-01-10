use super::ModeCursors;
use crate::{io::Io, modes};

pub async fn auto<T: Io>(pinger: &T, cursors: &mut ModeCursors) -> eyre::Result<()> {
    modes::discovery(pinger, &mut cursors.discovery).await
}

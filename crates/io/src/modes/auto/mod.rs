use super::ModeCursors;
use crate::{modes, Io};

pub async fn auto<T: Io>(pinger: &mut T, cursors: &mut ModeCursors) -> eyre::Result<()> {
    modes::discovery(pinger, &mut cursors.discovery).await
}

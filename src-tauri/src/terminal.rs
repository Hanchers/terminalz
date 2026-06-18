use anyhow::Result;
use tokio::sync::{mpsc, Mutex};

/// Shared command type for terminal sessions (SSH and local PTY).
pub(crate) enum Command {
    Write(Vec<u8>),
    Resize(u32, u32),
}

/// Wraps an mpsc sender behind a Mutex, providing write/resize/close for
/// terminal sessions. Both SSH and local terminals use identical channel
/// mechanics; this struct avoids duplicating that logic.
pub(crate) struct Channel {
    tx: Mutex<Option<mpsc::UnboundedSender<Command>>>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            tx: Mutex::new(None),
        }
    }

    pub async fn set(&self, sender: mpsc::UnboundedSender<Command>) {
        *self.tx.lock().await = Some(sender);
    }

    pub async fn write(&self, data: &[u8]) -> Result<()> {
        let tx = self.tx.lock().await;
        match &*tx {
            Some(tx) => tx
                .send(Command::Write(data.to_vec()))
                .map_err(|_| anyhow::anyhow!("session closed")),
            None => Err(anyhow::anyhow!("not connected")),
        }
    }

    pub async fn resize(&self, rows: u32, cols: u32) -> Result<()> {
        let tx = self.tx.lock().await;
        match &*tx {
            Some(tx) => tx
                .send(Command::Resize(rows, cols))
                .map_err(|_| anyhow::anyhow!("session closed")),
            None => Err(anyhow::anyhow!("not connected")),
        }
    }

    /// Drops the sender so that the IO loop exits naturally.
    pub async fn close(&self) {
        *self.tx.lock().await = None;
    }
}

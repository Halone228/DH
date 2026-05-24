//! Periodic SQLite backup via WAL checkpoint + file copy.

use sqlx::SqlitePool;
use std::time::Duration;
use tracing::{debug, error, info};

/// Periodically checkpoints the WAL and copies the database file as a backup.
pub struct SqliteBackup {
    pool: SqlitePool,
    db_path: String,
    interval: Duration,
}

impl SqliteBackup {
    pub fn new(pool: SqlitePool, db_path: String, interval: Duration) -> Self {
        Self {
            pool,
            db_path,
            interval,
        }
    }

    /// Run a WAL checkpoint (TRUNCATE mode). Returns the number of pages
    /// checkpointed.
    pub async fn checkpoint(&self) -> Result<u64, sqlx::Error> {
        let row: (i32, i32, i32) =
            sqlx::query_as("PRAGMA wal_checkpoint(TRUNCATE)")
                .fetch_one(&self.pool)
                .await?;
        Ok(row.1 as u64)
    }

    /// Run the backup loop until shutdown is signalled.
    pub async fn run_loop(
        self,
        mut shutdown: tokio::sync::broadcast::Receiver<()>,
    ) {
        let mut tick = tokio::time::interval(self.interval);
        loop {
            tokio::select! {
                _ = tick.tick() => {
                    match self.checkpoint().await {
                        Ok(pages) => debug!(pages, "wal checkpoint completed"),
                        Err(e) => error!(error = %e, "wal checkpoint failed"),
                    }
                    let backup_path = format!("{}.bak", self.db_path);
                    match tokio::fs::copy(&self.db_path, &backup_path).await {
                        Ok(_) => debug!("backup copy created"),
                        Err(e) => error!(error = %e, "backup copy failed"),
                    }
                }
                _ = shutdown.recv() => {
                    info!("backup loop shutting down");
                    break;
                }
            }
        }
    }
}

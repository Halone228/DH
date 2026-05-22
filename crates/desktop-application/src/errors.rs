use dayhelper_desktop_ports::{NotifyError, RepoError, SyncError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DesktopError {
    #[error("not authenticated — run `dayhelper-cli login <code>`")]
    NotAuthenticated,
    #[error("storage: {0}")]
    Storage(#[from] RepoError),
    #[error("sync: {0}")]
    Sync(#[from] SyncError),
    #[error("notify: {0}")]
    Notify(#[from] NotifyError),
    #[error("invalid input: {0}")]
    Invalid(String),
}

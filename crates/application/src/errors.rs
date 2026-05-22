use dayhelper_ports::{NotifyError, RepoError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("storage: {0}")]
    Storage(#[from] RepoError),
    #[error("notify: {0}")]
    Notify(#[from] NotifyError),
}

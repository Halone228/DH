use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("not found")]
    NotFound,
    #[error("storage error: {0}")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl RepoError {
    pub fn storage<E>(e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Storage(Box::new(e))
    }
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("not authenticated")]
    Unauthenticated,
    #[error("server rejected the request: {status} {message}")]
    Rejected { status: u16, message: String },
    #[error("network: {0}")]
    Transport(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, Error)]
pub enum NotifyError {
    #[error("desktop notification bus unavailable")]
    BusUnavailable,
    #[error("notify transport: {0}")]
    Transport(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, Error)]
pub enum TrackerError {
    #[error("compositor missing required protocol: {0}")]
    UnsupportedCompositor(&'static str),
    #[error("tracker io: {0}")]
    Io(#[source] Box<dyn std::error::Error + Send + Sync>),
}

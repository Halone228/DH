use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("entity not found")]
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
pub enum NotifyError {
    #[error("user is not reachable")]
    Unreachable,
    #[error("notifier transport error: {0}")]
    Transport(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl NotifyError {
    pub fn transport<E>(e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Transport(Box::new(e))
    }
}

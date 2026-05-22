use async_trait::async_trait;
use dayhelper_desktop_domain::Credentials;

use crate::errors::RepoError;

/// Persists the device's credentials. File-backed implementations write to
/// `~/.config/dayhelper/credentials.toml` with mode 600; in-memory ones are
/// for tests.
#[async_trait]
pub trait CredentialsStore: Send + Sync {
    async fn load(&self) -> Result<Option<Credentials>, RepoError>;
    async fn save(&self, creds: &Credentials) -> Result<(), RepoError>;
    async fn clear(&self) -> Result<(), RepoError>;
}

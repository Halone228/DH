//! TOML-on-disk credentials store.
//!
//! Path: `$XDG_CONFIG_HOME/dayhelper/credentials.toml` (or
//! `~/.config/dayhelper/credentials.toml`). On save, file mode is set to
//! 0600 so it's not world-readable.

use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

use async_trait::async_trait;
use dayhelper_desktop_domain::Credentials;
use dayhelper_desktop_ports::{CredentialsStore, RepoError};
use directories::ProjectDirs;

pub struct FileCredentialsStore {
    path: PathBuf,
}

impl FileCredentialsStore {
    /// `path` overrides the default location (intended for tests). Use
    /// [`Self::default_path`] for the production location.
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> Result<PathBuf, RepoError> {
        let dirs = ProjectDirs::from("dev", "dayhelper", "dayhelper").ok_or_else(|| {
            RepoError::Storage("cannot resolve project dirs (no $HOME?)".into())
        })?;
        Ok(dirs.config_dir().join("credentials.toml"))
    }

    pub fn default_paths() -> Result<Self, RepoError> {
        Ok(Self::at(Self::default_path()?))
    }
}

#[async_trait]
impl CredentialsStore for FileCredentialsStore {
    async fn load(&self) -> Result<Option<Credentials>, RepoError> {
        let path = self.path.clone();
        let bytes = tokio::task::spawn_blocking(move || -> Result<Option<Vec<u8>>, std::io::Error> {
            match std::fs::read(&path) {
                Ok(b) => Ok(Some(b)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e),
            }
        })
        .await
        .map_err(|e| RepoError::Storage(Box::new(e)))?
        .map_err(RepoError::storage)?;

        let Some(bytes) = bytes else {
            return Ok(None);
        };
        let s = String::from_utf8(bytes).map_err(RepoError::storage)?;
        let creds: Credentials = toml::from_str(&s).map_err(RepoError::storage)?;
        Ok(Some(creds))
    }

    async fn save(&self, creds: &Credentials) -> Result<(), RepoError> {
        let serialised = toml::to_string_pretty(creds).map_err(RepoError::storage)?;
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || -> Result<(), std::io::Error> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)?;
            use std::io::Write;
            f.write_all(serialised.as_bytes())?;
            f.sync_all()?;
            Ok(())
        })
        .await
        .map_err(|e| RepoError::Storage(Box::new(e)))?
        .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), RepoError> {
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || -> Result<(), std::io::Error> {
            match std::fs::remove_file(&path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e),
            }
        })
        .await
        .map_err(|e| RepoError::Storage(Box::new(e)))?
        .map_err(RepoError::storage)?;
        Ok(())
    }
}

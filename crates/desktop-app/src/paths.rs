use std::path::PathBuf;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

pub struct DesktopPaths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl DesktopPaths {
    pub fn discover() -> Result<Self> {
        let dirs = ProjectDirs::from("dev", "dayhelper", "dayhelper")
            .ok_or_else(|| anyhow!("cannot resolve user dirs (no $HOME?)"))?;
        Ok(Self {
            data_dir: dirs.data_dir().to_path_buf(),
            config_dir: dirs.config_dir().to_path_buf(),
        })
    }

    pub fn db_url(&self) -> Result<String> {
        std::fs::create_dir_all(&self.data_dir)?;
        let path = self.data_dir.join("local.db");
        Ok(format!("sqlite://{}", path.display()))
    }

    pub fn credentials_path(&self) -> PathBuf {
        self.config_dir.join("credentials.toml")
    }
}

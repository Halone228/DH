use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_domain::UserId;
use serde::{Deserialize, Serialize};

use crate::errors::RepoError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopActivityRow {
    pub id: uuid::Uuid,
    pub user_id: UserId,
    pub app_name: String,
    pub window_title: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
}

#[async_trait]
pub trait DesktopActivityRepo: Send + Sync {
    async fn append_batch(&self, rows: &[DesktopActivityRow]) -> Result<(), RepoError>;

    /// Delete rows with `received_at < threshold`. Returns count removed.
    async fn prune_before(&self, threshold: DateTime<Utc>) -> Result<u64, RepoError>;
}

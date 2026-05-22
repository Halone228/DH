use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_domain::{DesktopToken, DesktopTokenId, UserId};

use crate::errors::RepoError;

#[async_trait]
pub trait DesktopTokenRepo: Send + Sync {
    async fn insert(&self, token: &DesktopToken) -> Result<(), RepoError>;
    async fn find_active_by_hash(&self, hash: &str) -> Result<Option<DesktopToken>, RepoError>;
    async fn touch_last_seen(
        &self,
        id: DesktopTokenId,
        at: DateTime<Utc>,
    ) -> Result<(), RepoError>;
    async fn revoke(&self, id: DesktopTokenId) -> Result<(), RepoError>;
    async fn list_active_for_user(&self, user_id: UserId)
        -> Result<Vec<DesktopToken>, RepoError>;
}

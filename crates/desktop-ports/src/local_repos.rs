use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_desktop_domain::{ActivityEvent, LocalNotification, LocalNotificationState};
use uuid::Uuid;

use crate::errors::RepoError;

#[async_trait]
pub trait LocalActivityRepo: Send + Sync {
    async fn append(&self, event: &ActivityEvent) -> Result<(), RepoError>;
    async fn unsynced(&self, limit: u32) -> Result<Vec<ActivityEvent>, RepoError>;
    async fn mark_synced(&self, ids: &[Uuid]) -> Result<(), RepoError>;
    async fn prune_synced_before(&self, threshold: DateTime<Utc>) -> Result<u64, RepoError>;
}

#[async_trait]
pub trait LocalNotificationRepo: Send + Sync {
    async fn upsert(&self, n: &LocalNotification) -> Result<(), RepoError>;
    async fn pending_due(&self, now: DateTime<Utc>) -> Result<Vec<LocalNotification>, RepoError>;
    async fn mark(
        &self,
        id: Uuid,
        state: LocalNotificationState,
        fired_at: Option<DateTime<Utc>>,
    ) -> Result<(), RepoError>;
    /// Returns IDs that were `Fired` since the last call to
    /// `clear_fired_acks` — used to ack to the server on next sync.
    async fn fired_pending_ack(&self) -> Result<Vec<Uuid>, RepoError>;
    async fn clear_fired_acks(&self, ids: &[Uuid]) -> Result<(), RepoError>;
}

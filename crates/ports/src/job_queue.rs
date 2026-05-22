use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_domain::{JobId, ReminderId, UserId};
use serde::{Deserialize, Serialize};

use crate::errors::RepoError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind {
    /// Fire a stored `Reminder`. After firing, the scheduler computes the
    /// next occurrence (if recurring) and enqueues a new job.
    Reminder { reminder_id: ReminderId },
    /// Anti-procrastination nudge with a specific message.
    Nudge { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: JobId,
    pub user_id: UserId,
    pub kind: JobKind,
    pub fire_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Persistent job queue. Built around polling rather than push so it can be
/// implemented on plain SQL — no LISTEN/NOTIFY assumptions, no Redis required.
#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue(&self, job: ScheduledJob) -> Result<(), RepoError>;

    /// Earliest unfired job whose `fire_at <= now`. Implementations should
    /// claim the row atomically (e.g. `UPDATE ... RETURNING`) so two scheduler
    /// instances don't fire the same job twice.
    async fn pop_due(&self, now: DateTime<Utc>) -> Result<Option<ScheduledJob>, RepoError>;

    /// `fire_at` of the soonest pending job, or `None` if the queue is empty.
    /// Lets the scheduler sleep until the next event instead of polling.
    async fn peek_next_fire_at(&self) -> Result<Option<DateTime<Utc>>, RepoError>;

    /// All pending jobs for one user with `fire_at <= until`. Used by the
    /// desktop sync endpoint to mirror the next batch of notifications to
    /// a paired client.
    async fn pending_for_user_until(
        &self,
        user_id: UserId,
        until: DateTime<Utc>,
    ) -> Result<Vec<ScheduledJob>, RepoError>;

    /// Count of unfired nudges for one user with `start <= fire_at < end`.
    /// Used by `ScheduleDailyNudges` to keep itself idempotent — if today's
    /// nudges are already in the queue, the use case skips planning instead
    /// of doubling them.
    async fn count_pending_nudges_in_window(
        &self,
        user_id: UserId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<u64, RepoError>;

    /// Delete `fired_at IS NOT NULL AND fired_at < threshold` rows.
    /// Returns count of removed rows. Called by the daily prune loop.
    async fn prune_fired_before(&self, threshold: DateTime<Utc>) -> Result<u64, RepoError>;

    /// Drop all unfired jobs of a given kind for one user. Used when a
    /// reminder is cancelled or nudge settings change.
    async fn cancel_for_reminder(&self, reminder_id: ReminderId) -> Result<(), RepoError>;
    async fn cancel_nudges_for_user(&self, user_id: UserId) -> Result<(), RepoError>;
}

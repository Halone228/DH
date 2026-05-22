use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalNotificationState {
    Pending,
    /// Successfully shown via the desktop notifier.
    Fired,
    /// Skipped because too late (e.g. nudge whose `fire_at` passed long ago
    /// while the daemon was offline). Kept in history for diagnostics.
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNotification {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub fire_at: DateTime<Utc>,
    pub category: String,
    pub state: LocalNotificationState,
    pub fired_at: Option<DateTime<Utc>>,
}

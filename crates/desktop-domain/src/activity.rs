use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActivityEventId(pub Uuid);

impl ActivityEventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ActivityEventId {
    fn default() -> Self {
        Self::new()
    }
}

/// One closed activity session. Constructed by the application layer when
/// focus changes (or idle starts) — each event represents a continuous
/// interval where one app was actively in focus and the user was not idle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub id: ActivityEventId,
    pub app_name: String,
    pub window_title: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub synced: bool,
}

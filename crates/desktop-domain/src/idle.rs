use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User idle state. `idle_since == None` means "currently active".
/// We split activity sessions on idle/active transitions so AFK time
/// doesn't accumulate against whatever was last focused.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IdleStatus {
    pub idle_since: Option<DateTime<Utc>>,
}

impl IdleStatus {
    pub const fn active() -> Self {
        Self { idle_since: None }
    }

    pub const fn idle(at: DateTime<Utc>) -> Self {
        Self { idle_since: Some(at) }
    }

    pub const fn is_idle(&self) -> bool {
        self.idle_since.is_some()
    }
}

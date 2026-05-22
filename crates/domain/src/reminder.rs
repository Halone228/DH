use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{ReminderId, UserId};
use crate::recurrence::Recurrence;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: ReminderId,
    pub user_id: UserId,
    pub text: String,
    pub recurrence: Recurrence,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

impl Reminder {
    pub fn new(user_id: UserId, text: String, recurrence: Recurrence, now: DateTime<Utc>) -> Self {
        Self {
            id: ReminderId::new(),
            user_id,
            text,
            recurrence,
            active: true,
            created_at: now,
        }
    }
}

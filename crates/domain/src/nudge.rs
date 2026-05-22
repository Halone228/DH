use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use crate::ids::UserId;

pub const DEFAULT_NUDGE_COUNT: u8 = 5;

/// Per-user configuration for the anti-procrastination nudges.
///
/// `daily_count` random reminders are scheduled inside the
/// `[active_window_start, active_window_end)` window (user-local time)
/// each day. Defaults satisfy the original product spec: 5 nudges per day
/// between 09:00 and 21:00 local time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NudgeSettings {
    pub user_id: UserId,
    pub enabled: bool,
    pub daily_count: u8,
    pub active_window_start: NaiveTime,
    pub active_window_end: NaiveTime,
}

impl NudgeSettings {
    pub fn default_for(user_id: UserId) -> Self {
        Self {
            user_id,
            enabled: true,
            daily_count: DEFAULT_NUDGE_COUNT,
            active_window_start: NaiveTime::from_hms_opt(9, 0, 0).expect("valid"),
            active_window_end: NaiveTime::from_hms_opt(21, 0, 0).expect("valid"),
        }
    }
}

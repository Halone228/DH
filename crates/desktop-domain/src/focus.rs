use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One discrete focus transition reported by the window tracker.
/// Special-cased: `app_name` is `None` when nothing is focused (e.g. the
/// user closed every window or switched to an empty workspace).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusChange {
    pub at: DateTime<Utc>,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
}

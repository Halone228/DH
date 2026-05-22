use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Bearer token issued by the server in exchange for a pair-code. Wrapped
/// so that no log statement can accidentally print it via `Display`.
#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceToken(String);

impl DeviceToken {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for DeviceToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeviceToken(***)")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub user_id: Uuid,
    pub server_url: String,
    pub token: DeviceToken,
    pub paired_at: DateTime<Utc>,
}

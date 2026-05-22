use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ids::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DesktopTokenId(pub Uuid);

impl DesktopTokenId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DesktopTokenId {
    fn default() -> Self {
        Self::new()
    }
}

/// Server-side record of a paired desktop client.
///
/// **Plaintext token never stored** — only its SHA-256 hex digest.
/// `label` is whatever the user passed to `dayhelper-cli login --label`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopToken {
    pub id: DesktopTokenId,
    pub user_id: UserId,
    pub token_hash: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl DesktopToken {
    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none()
    }
}

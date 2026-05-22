use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Internal user identity. Independent of Telegram so the model can later be
/// extended with users that arrived through other channels (web, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

/// Telegram user id as delivered by the Bot API. Wrapped to avoid mixing it
/// up with internal `UserId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TelegramUserId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReminderId(pub Uuid);

impl ReminderId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ReminderId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Uuid);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

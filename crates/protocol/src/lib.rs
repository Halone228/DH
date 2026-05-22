//! Wire protocol shared by the desktop client and the server.
//!
//! Versioned via the `Cargo.toml` of this crate — bump the major when the
//! shape changes incompatibly. The desktop client always sends its protocol
//! version in the `User-Agent`-style header `X-Dayhelper-Proto` so the
//! server can refuse mismatched clients with a clear error.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROTOCOL_VERSION: &str = "1";

/// Pairing flow:
///   1. user types `/pair` in the bot — server emits a short numeric code,
///      stores `code -> telegram_user, expires_at` in memory.
///   2. user runs `dayhelper-cli login <code>` — desktop POSTs this
///      to `/api/desktop/pair`.
///   3. server validates, mints a long-lived token bound to that user.
#[derive(Debug, Serialize, Deserialize)]
pub struct PairRequest {
    pub code: String,
    pub device_label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairResponse {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
}

/// One observed focus session: the user had `app_name` focused for some
/// continuous interval, with optional title. Sessions interrupted by idle
/// are split at the idle boundary by the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityBatchItem {
    pub app_name: String,
    pub window_title: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub since_cursor: Option<String>,
    pub activity: Vec<ActivityBatchItem>,
    pub fired_notifications: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub cursor: String,
    /// Notifications scheduled for the near future. Client persists them
    /// locally and fires via libnotify when due.
    pub notifications: Vec<NotificationDelivery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDelivery {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub fire_at: DateTime<Utc>,
    pub category: NotificationCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationCategory {
    Reminder,
    Nudge,
}

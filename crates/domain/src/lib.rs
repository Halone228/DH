//! Pure domain types. No I/O, no async, no framework dependencies.
//!
//! Anything in this crate must be deterministically testable without mocks.

pub mod desktop_token;
pub mod ids;
pub mod nudge;
pub mod recurrence;
pub mod reminder;
pub mod user;

pub use desktop_token::{DesktopToken, DesktopTokenId};
pub use ids::{JobId, ReminderId, TelegramUserId, UserId};
pub use nudge::{NudgeSettings, DEFAULT_NUDGE_COUNT};
pub use recurrence::{Recurrence, Weekday};
pub use reminder::Reminder;
pub use user::User;

//! Application layer — orchestrates domain types via ports.
//!
//! Each use case is a struct that owns its dependencies as `Arc<dyn Port>`.
//! Constructor injection only — no global state, no service locators.
//!
//! Add a new use case by:
//!   1. creating a new module with a struct that takes the ports it needs;
//!   2. exposing it from `lib.rs`;
//!   3. constructing it once in `app::container` and injecting it where needed.

pub mod errors;
pub mod messages;
pub mod use_cases;

pub use errors::AppError;
pub use use_cases::accept_desktop_sync::AcceptDesktopSync;
pub use use_cases::cancel_reminder::CancelReminder;
pub use use_cases::create_reminder::{CreateReminder, CreateReminderCommand};
pub use use_cases::ensure_user::EnsureUser;
pub use use_cases::fire_due_jobs::FireDueJobs;
pub use use_cases::issue_pair_code::IssuePairCode;
pub use use_cases::list_reminders::ListReminders;
pub use use_cases::prune_old_data::{PruneOldData, PruneRetention, PruneSummary};
pub use use_cases::redeem_pair_code::{RedeemPairCode, RedeemPairCodeOutcome};
pub use use_cases::schedule_nudges::ScheduleDailyNudges;
pub use use_cases::update_nudge_settings::UpdateNudgeSettings;
pub use use_cases::update_timezone::UpdateTimezone;

pub mod auth {
    //! Re-exports for the bearer-auth path used by `server-desktop-api`.
    pub use crate::use_cases::redeem_pair_code::sha256_hex;
}

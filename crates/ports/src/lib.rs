//! Ports — abstract boundaries between the application core and the outside
//! world. Implementations live in `adapter-*` crates.
//!
//! Every trait here is `Send + Sync` so its trait object is safe to share
//! across tokio tasks via `Arc<dyn Trait>`.

pub mod clock;
pub mod desktop_activity_repo;
pub mod desktop_token_repo;
pub mod errors;
pub mod job_queue;
pub mod notifier;
pub mod pair_code_store;
pub mod random;
pub mod reminder_repo;
pub mod user_repo;

pub use clock::Clock;
pub use desktop_activity_repo::{DesktopActivityRepo, DesktopActivityRow};
pub use desktop_token_repo::DesktopTokenRepo;
pub use errors::{NotifyError, RepoError};
pub use job_queue::{JobKind, JobQueue, ScheduledJob};
pub use notifier::Notifier;
pub use pair_code_store::PairCodeStore;
pub use random::RandomSource;
pub use reminder_repo::ReminderRepo;
pub use user_repo::{NudgeSettingsRepo, UserRepo};

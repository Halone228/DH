//! Ports for the desktop client. Concrete implementations live in
//! `dayhelper-desktop-adapter-*` crates.

pub mod credentials_store;
pub mod errors;
pub mod local_repos;
pub mod notifier;
pub mod sync_client;
pub mod tracker;

pub use credentials_store::CredentialsStore;
pub use errors::{NotifyError, RepoError, SyncError, TrackerError};
pub use local_repos::{LocalActivityRepo, LocalNotificationRepo};
pub use notifier::DesktopNotifier;
pub use sync_client::SyncClient;
pub use tracker::{IdleDetector, WindowTracker};

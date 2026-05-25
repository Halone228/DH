//! Desktop client use cases. Each one owns its ports as `Arc<dyn Port>`.

pub mod errors;
pub mod messages;
pub mod use_cases;

pub use errors::DesktopError;
pub use messages::Messages;
pub use use_cases::fire_due::FireDueLocalNotifications;
pub use use_cases::pair::PairDevice;
pub use use_cases::session::SessionAggregator;
pub use use_cases::sync::SyncWithServer;

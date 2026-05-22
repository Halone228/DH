//! Pure domain types for the desktop client. Independent of any I/O.

pub mod activity;
pub mod credentials;
pub mod focus;
pub mod idle;
pub mod notification;

pub use activity::{ActivityEvent, ActivityEventId};
pub use credentials::{Credentials, DeviceToken};
pub use focus::FocusChange;
pub use idle::IdleStatus;
pub use notification::{LocalNotification, LocalNotificationState};

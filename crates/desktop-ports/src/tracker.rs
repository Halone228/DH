use async_trait::async_trait;
use dayhelper_desktop_domain::{FocusChange, IdleStatus};
use tokio::sync::mpsc;

use crate::errors::TrackerError;

/// Window-focus tracker. Implementation runs an event loop (Wayland
/// dispatcher, X11 select-loop, etc.) and emits one [`FocusChange`] per
/// transition into `tx`. Returns when the channel closes or the compositor
/// disconnects.
#[async_trait]
pub trait WindowTracker: Send + Sync {
    async fn run(&self, tx: mpsc::Sender<FocusChange>) -> Result<(), TrackerError>;
}

/// User-idle tracker. Same shape as `WindowTracker` — emits transitions.
#[async_trait]
pub trait IdleDetector: Send + Sync {
    async fn run(&self, tx: mpsc::Sender<IdleStatus>) -> Result<(), TrackerError>;
}

use std::sync::Arc;

use chrono::{Duration, Utc};
use dayhelper_desktop_domain::LocalNotificationState;
use dayhelper_desktop_ports::{DesktopNotifier, LocalNotificationRepo};
use tracing::{info, warn};

use crate::DesktopError;

/// Notifications older than this are skipped rather than fired late. This
/// prevents a 4 AM nudge from popping up when the daemon comes back from
/// suspend at 9 AM.
const STALE_AFTER: Duration = Duration::minutes(15);

pub struct FireDueLocalNotifications {
    repo: Arc<dyn LocalNotificationRepo>,
    notifier: Arc<dyn DesktopNotifier>,
}

impl FireDueLocalNotifications {
    pub fn new(repo: Arc<dyn LocalNotificationRepo>, notifier: Arc<dyn DesktopNotifier>) -> Self {
        Self { repo, notifier }
    }

    /// Pop each notification whose `fire_at <= now`, decide whether to fire
    /// or skip based on staleness, and update its state.
    pub async fn tick(&self) -> Result<u32, DesktopError> {
        let now = Utc::now();
        let due = self.repo.pending_due(now).await?;
        let mut fired = 0u32;

        for n in due {
            let age = now - n.fire_at;
            if age > STALE_AFTER {
                warn!(id = %n.id, age_secs = age.num_seconds(), "skipping stale notification");
                self.repo
                    .mark(n.id, LocalNotificationState::Skipped, Some(now))
                    .await?;
                continue;
            }

            match self.notifier.show(&n.title, &n.body).await {
                Ok(()) => {
                    self.repo
                        .mark(n.id, LocalNotificationState::Fired, Some(now))
                        .await?;
                    fired += 1;
                }
                Err(e) => {
                    warn!(id = %n.id, error = %e, "notify failed; will retry next tick");
                    // Leave state Pending so we retry next tick.
                }
            }
        }

        if fired > 0 {
            info!(fired, "fired desktop notifications");
        }
        Ok(fired)
    }
}

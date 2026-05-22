use std::sync::Arc;

use chrono::Duration;
use dayhelper_ports::{Clock, DesktopActivityRepo, JobQueue};
use tracing::info;

use crate::AppError;

#[derive(Debug, Clone, Copy)]
pub struct PruneRetention {
    /// How long fired jobs (`scheduled_jobs.fired_at`) live before deletion.
    pub fired_jobs: Duration,
    /// How long desktop activity rows live before deletion.
    pub desktop_activity: Duration,
}

impl Default for PruneRetention {
    fn default() -> Self {
        Self {
            fired_jobs: Duration::days(30),
            desktop_activity: Duration::days(90),
        }
    }
}

#[derive(Debug, Default)]
pub struct PruneSummary {
    pub fired_jobs_removed: u64,
    pub desktop_activity_removed: u64,
}

/// Periodic cleanup. Safe to invoke any time — only deletes rows older than
/// the configured retention window.
pub struct PruneOldData {
    jobs: Arc<dyn JobQueue>,
    activity: Arc<dyn DesktopActivityRepo>,
    clock: Arc<dyn Clock>,
    retention: PruneRetention,
}

impl PruneOldData {
    pub fn new(
        jobs: Arc<dyn JobQueue>,
        activity: Arc<dyn DesktopActivityRepo>,
        clock: Arc<dyn Clock>,
        retention: PruneRetention,
    ) -> Self {
        Self {
            jobs,
            activity,
            clock,
            retention,
        }
    }

    pub async fn execute(&self) -> Result<PruneSummary, AppError> {
        let now = self.clock.now();
        let fired_jobs_removed = self
            .jobs
            .prune_fired_before(now - self.retention.fired_jobs)
            .await?;
        let desktop_activity_removed = self
            .activity
            .prune_before(now - self.retention.desktop_activity)
            .await?;
        let summary = PruneSummary {
            fired_jobs_removed,
            desktop_activity_removed,
        };
        info!(
            fired_jobs = summary.fired_jobs_removed,
            desktop_activity = summary.desktop_activity_removed,
            "prune complete"
        );
        Ok(summary)
    }
}

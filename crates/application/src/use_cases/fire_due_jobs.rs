use std::sync::Arc;

use chrono_tz::Tz;
use dayhelper_domain::{JobId, ReminderId};
use dayhelper_ports::{
    Clock, JobKind, JobQueue, Notifier, ReminderRepo, ScheduledJob, UserRepo,
};
use tracing::{error, warn};

/// Nudges older than this are skipped rather than fired late. This prevents
/// a 4 AM nudge from popping up when the server comes back up at 9 AM.
/// Reminders are NOT affected — they fire late by design.
const NUDGE_STALE_THRESHOLD: chrono::Duration = chrono::Duration::minutes(30);

use crate::AppError;

/// One step of the scheduler loop. The runtime crate calls this in a tight
/// loop with a `tokio::time::sleep_until` between calls.
pub struct FireDueJobs {
    jobs: Arc<dyn JobQueue>,
    reminders: Arc<dyn ReminderRepo>,
    users: Arc<dyn UserRepo>,
    notifier: Arc<dyn Notifier>,
    clock: Arc<dyn Clock>,
}

impl FireDueJobs {
    pub fn new(
        jobs: Arc<dyn JobQueue>,
        reminders: Arc<dyn ReminderRepo>,
        users: Arc<dyn UserRepo>,
        notifier: Arc<dyn Notifier>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            jobs,
            reminders,
            users,
            notifier,
            clock,
        }
    }

    /// Pop and fire one due job. Returns:
    ///  - `Ok(Some(job))` if a job fired (caller should immediately try again);
    ///  - `Ok(None)` if no due job is currently waiting.
    pub async fn tick(&self) -> Result<Option<ScheduledJob>, AppError> {
        let now = self.clock.now();
        let Some(job) = self.jobs.pop_due(now).await? else {
            return Ok(None);
        };
        self.handle(&job).await?;
        Ok(Some(job))
    }

    async fn handle(&self, job: &ScheduledJob) -> Result<(), AppError> {
        let Some(user) = self.users.find_by_id(job.user_id).await? else {
            warn!(user = ?job.user_id, "job for unknown user, dropping");
            return Ok(());
        };

        match &job.kind {
            JobKind::Nudge { message } => {
                let age = self.clock.now() - job.fire_at;
                if age > NUDGE_STALE_THRESHOLD {
                    warn!(
                        job = %job.id.0,
                        age_secs = age.num_seconds(),
                        "skipping stale nudge"
                    );
                    return Ok(());
                }
                self.notifier.notify(user.telegram_id, message).await?;
            }
            JobKind::Reminder { reminder_id } => {
                self.fire_reminder(*reminder_id, user.telegram_id, user.timezone)
                    .await?;
            }
        }
        Ok(())
    }

    async fn fire_reminder(
        &self,
        id: ReminderId,
        telegram_id: dayhelper_domain::TelegramUserId,
        tz: Tz,
    ) -> Result<(), AppError> {
        let Some(reminder) = self.reminders.get(id).await? else {
            warn!(reminder = ?id, "reminder vanished before firing");
            return Ok(());
        };
        if !reminder.active {
            return Ok(());
        }

        if let Err(e) = self.notifier.notify(telegram_id, &reminder.text).await {
            error!(error = %e, "failed to deliver reminder");
            // The reminder has been popped from the queue; we still re-enqueue
            // the next occurrence so future fires keep working.
        }

        if let Some(next_at) = reminder.recurrence.next_after(self.clock.now(), tz) {
            self.jobs
                .enqueue(ScheduledJob {
                    id: JobId::new(),
                    user_id: reminder.user_id,
                    kind: JobKind::Reminder {
                        reminder_id: reminder.id,
                    },
                    fire_at: next_at,
                    created_at: self.clock.now(),
                })
                .await?;
        } else {
            // Once-only reminder finished — deactivate.
            self.reminders.deactivate(reminder.id).await?;
        }
        Ok(())
    }

}

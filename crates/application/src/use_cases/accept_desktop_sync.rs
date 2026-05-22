use std::sync::Arc;

use chrono::Duration;
use dayhelper_domain::{Reminder, ReminderId, User};
use dayhelper_ports::{
    Clock, DesktopActivityRepo, DesktopActivityRow, JobKind, JobQueue, ReminderRepo, ScheduledJob,
};
use dayhelper_protocol::{
    ActivityBatchItem, NotificationCategory, NotificationDelivery, SyncRequest, SyncResponse,
};
use tracing::debug;
use uuid::Uuid;

use crate::AppError;

/// Window of upcoming notifications returned to the desktop client. The
/// client polls every minute, so an hour gives plenty of headroom for
/// brief offline periods.
const LOOKAHEAD: Duration = Duration::hours(1);

pub struct AcceptDesktopSync {
    activity: Arc<dyn DesktopActivityRepo>,
    jobs: Arc<dyn JobQueue>,
    reminders: Arc<dyn ReminderRepo>,
    clock: Arc<dyn Clock>,
}

impl AcceptDesktopSync {
    pub fn new(
        activity: Arc<dyn DesktopActivityRepo>,
        jobs: Arc<dyn JobQueue>,
        reminders: Arc<dyn ReminderRepo>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            activity,
            jobs,
            reminders,
            clock,
        }
    }

    pub async fn execute(
        &self,
        user: &User,
        req: SyncRequest,
    ) -> Result<SyncResponse, AppError> {
        let now = self.clock.now();

        if !req.activity.is_empty() {
            let rows: Vec<DesktopActivityRow> = req
                .activity
                .iter()
                .map(|a| activity_row(user, a, now))
                .collect();
            self.activity.append_batch(&rows).await?;
        }

        if !req.fired_notifications.is_empty() {
            // Currently informational. A future enhancement: a
            // `desktop_notification_deliveries` table for per-device stats.
            debug!(
                count = req.fired_notifications.len(),
                "desktop acked fired notifications"
            );
        }

        let until = now + LOOKAHEAD;
        let upcoming = self.jobs_for_user_in_window(user, until).await?;
        let notifications = self.materialize(upcoming).await?;

        Ok(SyncResponse {
            cursor: now.to_rfc3339(),
            notifications,
        })
    }

    async fn jobs_for_user_in_window(
        &self,
        user: &User,
        until: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<ScheduledJob>, AppError> {
        Ok(self.jobs.pending_for_user_until(user.id, until).await?)
    }

    async fn materialize(
        &self,
        jobs: Vec<ScheduledJob>,
    ) -> Result<Vec<NotificationDelivery>, AppError> {
        let mut out = Vec::with_capacity(jobs.len());
        for job in jobs {
            let delivery = match &job.kind {
                JobKind::Reminder { reminder_id } => {
                    match self.fetch_reminder(*reminder_id).await? {
                        Some(r) => Some(NotificationDelivery {
                            id: job.id.0,
                            title: "Напоминание".to_string(),
                            body: r.text,
                            fire_at: job.fire_at,
                            category: NotificationCategory::Reminder,
                        }),
                        None => None,
                    }
                }
                JobKind::Nudge { message } => Some(NotificationDelivery {
                    id: job.id.0,
                    title: "Не прокрастинируй".to_string(),
                    body: message.clone(),
                    fire_at: job.fire_at,
                    category: NotificationCategory::Nudge,
                }),
            };
            if let Some(d) = delivery {
                out.push(d);
            }
        }
        Ok(out)
    }

    async fn fetch_reminder(&self, id: ReminderId) -> Result<Option<Reminder>, AppError> {
        Ok(self.reminders.get(id).await?)
    }
}

fn activity_row(
    user: &User,
    item: &ActivityBatchItem,
    now: chrono::DateTime<chrono::Utc>,
) -> DesktopActivityRow {
    DesktopActivityRow {
        id: Uuid::new_v4(),
        user_id: user.id,
        app_name: item.app_name.clone(),
        window_title: item.window_title.clone(),
        started_at: item.started_at,
        ended_at: item.ended_at,
        received_at: now,
    }
}

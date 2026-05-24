use std::sync::Arc;

use dayhelper_domain::ReminderId;
use dayhelper_ports::{JobQueue, ReminderRepo};

use crate::AppError;

pub struct CancelReminder {
    reminders: Arc<dyn ReminderRepo>,
    jobs: Arc<dyn JobQueue>,
}

impl CancelReminder {
    pub fn new(reminders: Arc<dyn ReminderRepo>, jobs: Arc<dyn JobQueue>) -> Self {
        Self { reminders, jobs }
    }

    pub async fn execute(&self, id: ReminderId) -> Result<(), AppError> {
        self.reminders.deactivate(id).await?;
        self.jobs.cancel_for_reminder(id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{FakeJobQueue, FakeReminderRepo};
    use chrono::{TimeZone, Utc};
    use dayhelper_domain::{Reminder, UserId};

    #[tokio::test]
    async fn test_cancel_existing_reminder() {
        let reminders = Arc::new(FakeReminderRepo::new());
        let jobs = Arc::new(FakeJobQueue::new());
        let uc = CancelReminder::new(reminders.clone(), jobs.clone());

        let r = Reminder::new(
            UserId::new(),
            "test".into(),
            dayhelper_domain::Recurrence::Once {
                at: Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap(),
            },
            Utc::now(),
        );
        reminders.save(&r).await.unwrap();

        uc.execute(r.id).await.unwrap();
        let stored = reminders.get(r.id).await.unwrap().unwrap();
        assert!(!stored.active);
    }

    #[tokio::test]
    async fn test_cancel_nonexistent_reminder() {
        let reminders = Arc::new(FakeReminderRepo::new());
        let jobs = Arc::new(FakeJobQueue::new());
        let uc = CancelReminder::new(reminders.clone(), jobs.clone());
        // FakeReminderRepo.deactivate is a no-op for missing IDs — should not error
        uc.execute(ReminderId::new()).await.unwrap();
    }

    #[tokio::test]
    async fn test_cancel_deactivates_and_cancels_jobs() {
        let reminders = Arc::new(FakeReminderRepo::new());
        let jobs = Arc::new(FakeJobQueue::new());
        let uc = CancelReminder::new(reminders.clone(), jobs.clone());

        let rid = ReminderId::new();
        let r = Reminder::new(
            UserId::new(),
            "test".into(),
            dayhelper_domain::Recurrence::Once {
                at: Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap(),
            },
            Utc::now(),
        );
        // Use the specific id
        let mut r2 = r;
        r2.id = rid;
        reminders.save(&r2).await.unwrap();

        // Enqueue a job for this reminder
        jobs.enqueue(dayhelper_ports::ScheduledJob {
            id: dayhelper_domain::JobId::new(),
            user_id: r2.user_id,
            kind: dayhelper_ports::JobKind::Reminder { reminder_id: rid },
            fire_at: Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap(),
            created_at: Utc::now(),
        })
        .await
        .unwrap();

        uc.execute(rid).await.unwrap();
        assert_eq!(jobs.len().await, 0);
    }
}

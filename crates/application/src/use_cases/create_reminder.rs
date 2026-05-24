use std::sync::Arc;

use chrono_tz::Tz;
use dayhelper_domain::{Recurrence, Reminder, UserId};
use dayhelper_ports::{Clock, JobKind, JobQueue, ReminderRepo, ScheduledJob};
use dayhelper_domain::JobId;

use crate::AppError;

pub struct CreateReminderCommand {
    pub user_id: UserId,
    pub user_timezone: Tz,
    pub text: String,
    pub recurrence: Recurrence,
}

pub struct CreateReminder {
    reminders: Arc<dyn ReminderRepo>,
    jobs: Arc<dyn JobQueue>,
    clock: Arc<dyn Clock>,
}

impl CreateReminder {
    pub fn new(
        reminders: Arc<dyn ReminderRepo>,
        jobs: Arc<dyn JobQueue>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            reminders,
            jobs,
            clock,
        }
    }

    pub async fn execute(&self, cmd: CreateReminderCommand) -> Result<Reminder, AppError> {
        if cmd.text.trim().is_empty() {
            return Err(AppError::Invalid("reminder text is empty".into()));
        }

        let now = self.clock.now();
        let reminder = Reminder::new(cmd.user_id, cmd.text, cmd.recurrence.clone(), now);

        let next = cmd
            .recurrence
            .next_after(now, cmd.user_timezone)
            .ok_or_else(|| AppError::Invalid("recurrence has no future occurrence".into()))?;

        self.reminders.save(&reminder).await?;
        self.jobs
            .enqueue(ScheduledJob {
                id: JobId::new(),
                user_id: cmd.user_id,
                kind: JobKind::Reminder {
                    reminder_id: reminder.id,
                },
                fire_at: next,
                created_at: now,
            })
            .await?;

        Ok(reminder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{FakeClock, FakeJobQueue, FakeReminderRepo};
    use chrono::{DateTime, TimeZone, Utc};
    use chrono_tz::Europe::Moscow;
    use dayhelper_domain::Weekday;

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 24, 12, 0, 0).unwrap()
    }

    fn make_uc() -> (
        Arc<FakeReminderRepo>,
        Arc<FakeJobQueue>,
        Arc<FakeClock>,
        CreateReminder,
    ) {
        let reminders = Arc::new(FakeReminderRepo::new());
        let jobs = Arc::new(FakeJobQueue::new());
        let clock = Arc::new(FakeClock::new(fixed_now()));
        let uc = CreateReminder::new(
            reminders.clone(),
            jobs.clone(),
            clock.clone(),
        );
        (reminders, jobs, clock, uc)
    }

    #[tokio::test]
    async fn test_create_once_reminder() {
        let (reminders, _, _, uc) = make_uc();
        let user_id = UserId::new();
        let at = fixed_now() + chrono::Duration::hours(1);
        let r = uc
            .execute(CreateReminderCommand {
                user_id,
                user_timezone: Moscow,
                text: "test".into(),
                recurrence: Recurrence::Once { at },
            })
            .await
            .unwrap();
        assert_eq!(r.text, "test");
        assert!(r.active);
        let stored = reminders.get(r.id).await.unwrap().unwrap();
        assert_eq!(stored.text, "test");
    }

    #[tokio::test]
    async fn test_create_daily_reminder() {
        let (_, _, _, uc) = make_uc();
        let r = uc
            .execute(CreateReminderCommand {
                user_id: UserId::new(),
                user_timezone: Moscow,
                text: "daily test".into(),
                recurrence: Recurrence::Daily {
                    time: chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                },
            })
            .await
            .unwrap();
        assert_eq!(r.text, "daily test");
    }

    #[tokio::test]
    async fn test_create_weekly_reminder() {
        let (_, _, _, uc) = make_uc();
        let r = uc
            .execute(CreateReminderCommand {
                user_id: UserId::new(),
                user_timezone: Moscow,
                text: "weekly test".into(),
                recurrence: Recurrence::Weekly {
                    weekdays: vec![Weekday::Mon, Weekday::Wed],
                    time: chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                },
            })
            .await
            .unwrap();
        assert_eq!(r.text, "weekly test");
    }

    #[tokio::test]
    async fn test_create_monthly_reminder() {
        let (_, _, _, uc) = make_uc();
        let r = uc
            .execute(CreateReminderCommand {
                user_id: UserId::new(),
                user_timezone: Moscow,
                text: "monthly test".into(),
                recurrence: Recurrence::Monthly {
                    day_of_month: 15,
                    time: chrono::NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                },
            })
            .await
            .unwrap();
        assert_eq!(r.text, "monthly test");
    }

    #[tokio::test]
    async fn test_create_reminder_empty_text_fails() {
        let (_, _, _, uc) = make_uc();
        let result = uc
            .execute(CreateReminderCommand {
                user_id: UserId::new(),
                user_timezone: Moscow,
                text: "   ".into(),
                recurrence: Recurrence::Once {
                    at: fixed_now() + chrono::Duration::hours(1),
                },
            })
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::AppError::Invalid(_)));
    }

    #[tokio::test]
    async fn test_create_reminder_enqueues_job() {
        let (_, jobs, _, uc) = make_uc();
        let user_id = UserId::new();
        uc.execute(CreateReminderCommand {
            user_id,
            user_timezone: Moscow,
            text: "test".into(),
            recurrence: Recurrence::Once {
                at: fixed_now() + chrono::Duration::hours(1),
            },
        })
        .await
        .unwrap();
        assert_eq!(jobs.len().await, 1);
    }
}

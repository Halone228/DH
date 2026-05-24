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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{FakeClock, FakeJobQueue, FakeNotifier, FakeReminderRepo, FakeUserRepo};
    use chrono::{DateTime, TimeZone, Utc};
    use chrono_tz::Europe::Moscow;
    use dayhelper_domain::{JobId, TelegramUserId, UserId};

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 24, 12, 0, 0).unwrap()
    }

    fn make_uc_with(clock: Arc<FakeClock>) -> (
        Arc<FakeJobQueue>,
        Arc<FakeReminderRepo>,
        Arc<FakeUserRepo>,
        Arc<FakeNotifier>,
        FireDueJobs,
    ) {
        let jobs = Arc::new(FakeJobQueue::new());
        let reminders = Arc::new(FakeReminderRepo::new());
        let users = Arc::new(FakeUserRepo::new());
        let notifier = Arc::new(FakeNotifier::new());
        let uc = FireDueJobs::new(
            jobs.clone(),
            reminders.clone(),
            users.clone(),
            notifier.clone(),
            clock.clone(),
        );
        (jobs, reminders, users, notifier, uc)
    }

    async fn seed_user(users: &FakeUserRepo) -> (UserId, TelegramUserId) {
        let tg = TelegramUserId(99);
        let user = dayhelper_domain::User::new(tg, Moscow);
        let uid = user.id;
        users.upsert(&user).await.unwrap();
        (uid, tg)
    }

    #[tokio::test]
    async fn test_fire_reminder_sends_notification() {
        let clock = Arc::new(FakeClock::new(fixed_now()));
        let (jobs, reminders, users, notifier, uc) = make_uc_with(clock.clone());
        let (uid, tg) = seed_user(&users).await;

        let reminder = dayhelper_domain::Reminder::new(
            uid,
            "hello".into(),
            dayhelper_domain::Recurrence::Once {
                at: fixed_now() - chrono::Duration::minutes(5),
            },
            fixed_now(),
        );
        let rid = reminder.id;
        reminders.save(&reminder).await.unwrap();

        jobs.enqueue(dayhelper_ports::ScheduledJob {
            id: JobId::new(),
            user_id: uid,
            kind: dayhelper_ports::JobKind::Reminder { reminder_id: rid },
            fire_at: fixed_now() - chrono::Duration::minutes(5),
            created_at: fixed_now(),
        })
        .await
        .unwrap();

        let result = uc.tick().await.unwrap();
        assert!(result.is_some());
        let sent = notifier.sent().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, tg);
        assert!(sent[0].1.contains("hello"));
    }

    #[tokio::test]
    async fn test_fire_nudge_sends_notification() {
        let clock = Arc::new(FakeClock::new(fixed_now()));
        let (jobs, _, users, notifier, uc) = make_uc_with(clock.clone());
        let (uid, tg) = seed_user(&users).await;

        jobs.enqueue(dayhelper_ports::ScheduledJob {
            id: JobId::new(),
            user_id: uid,
            kind: dayhelper_ports::JobKind::Nudge {
                message: "Do something!".into(),
            },
            fire_at: fixed_now() - chrono::Duration::minutes(5),
            created_at: fixed_now(),
        })
        .await
        .unwrap();

        let result = uc.tick().await.unwrap();
        assert!(result.is_some());
        let sent = notifier.sent().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, tg);
        assert_eq!(sent[0].1, "Do something!");
    }

    #[tokio::test]
    async fn test_stale_nudge_skipped() {
        let clock = Arc::new(FakeClock::new(fixed_now()));
        let (jobs, _, users, notifier, uc) = make_uc_with(clock.clone());
        let (uid, _) = seed_user(&users).await;

        // Nudge from 31 minutes ago — beyond the 30-min threshold
        jobs.enqueue(dayhelper_ports::ScheduledJob {
            id: JobId::new(),
            user_id: uid,
            kind: dayhelper_ports::JobKind::Nudge {
                message: "stale nudge".into(),
            },
            fire_at: fixed_now() - chrono::Duration::minutes(31),
            created_at: fixed_now(),
        })
        .await
        .unwrap();

        let result = uc.tick().await.unwrap();
        assert!(result.is_some());
        let sent = notifier.sent().await;
        assert!(sent.is_empty(), "stale nudge should be skipped");
    }

    #[tokio::test]
    async fn test_reminder_fires_regardless_of_age() {
        let clock = Arc::new(FakeClock::new(fixed_now()));
        let (jobs, reminders, users, notifier, uc) = make_uc_with(clock.clone());
        let (uid, _) = seed_user(&users).await;

        let at_time = fixed_now() - chrono::Duration::hours(2);
        let reminder = dayhelper_domain::Reminder::new(
            uid,
            "old reminder".into(),
            dayhelper_domain::Recurrence::Once { at: at_time },
            fixed_now(),
        );
        let rid = reminder.id;
        reminders.save(&reminder).await.unwrap();

        jobs.enqueue(dayhelper_ports::ScheduledJob {
            id: JobId::new(),
            user_id: uid,
            kind: dayhelper_ports::JobKind::Reminder { reminder_id: rid },
            fire_at: at_time,
            created_at: fixed_now(),
        })
        .await
        .unwrap();

        let result = uc.tick().await.unwrap();
        assert!(result.is_some());
        let sent = notifier.sent().await;
        assert_eq!(sent.len(), 1);
    }

    #[tokio::test]
    async fn test_once_reminder_deactivates_after_fire() {
        let clock = Arc::new(FakeClock::new(fixed_now()));
        let (jobs, reminders, users, _, uc) = make_uc_with(clock.clone());
        let (uid, _) = seed_user(&users).await;

        let at_time = fixed_now() - chrono::Duration::minutes(5);
        let reminder = dayhelper_domain::Reminder::new(
            uid,
            "once".into(),
            dayhelper_domain::Recurrence::Once { at: at_time },
            fixed_now(),
        );
        let rid = reminder.id;
        reminders.save(&reminder).await.unwrap();

        jobs.enqueue(dayhelper_ports::ScheduledJob {
            id: JobId::new(),
            user_id: uid,
            kind: dayhelper_ports::JobKind::Reminder { reminder_id: rid },
            fire_at: at_time,
            created_at: fixed_now(),
        })
        .await
        .unwrap();

        uc.tick().await.unwrap();

        let stored = reminders.get(rid).await.unwrap().unwrap();
        assert!(!stored.active, "once reminder should be deactivated after fire");
    }
}


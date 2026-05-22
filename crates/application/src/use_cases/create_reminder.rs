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

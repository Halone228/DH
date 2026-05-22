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

use std::sync::Arc;

use dayhelper_domain::{Reminder, UserId};
use dayhelper_ports::ReminderRepo;

use crate::AppError;

pub struct ListReminders {
    reminders: Arc<dyn ReminderRepo>,
}

impl ListReminders {
    pub fn new(reminders: Arc<dyn ReminderRepo>) -> Self {
        Self { reminders }
    }

    pub async fn execute(&self, user_id: UserId) -> Result<Vec<Reminder>, AppError> {
        Ok(self.reminders.list_for_user(user_id).await?)
    }
}

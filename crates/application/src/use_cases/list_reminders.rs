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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::FakeReminderRepo;
    use chrono::{TimeZone, Utc};
    use dayhelper_domain::Reminder;

    #[tokio::test]
    async fn test_list_empty() {
        let repo = Arc::new(FakeReminderRepo::new());
        let uc = ListReminders::new(repo);
        let uid = UserId::new();
        let result = uc.execute(uid).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_with_items() {
        let repo = Arc::new(FakeReminderRepo::new());
        let uid = UserId::new();

        let r1 = Reminder::new(
            uid,
            "first".into(),
            dayhelper_domain::Recurrence::Once {
                at: Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap(),
            },
            Utc::now(),
        );
        let r2 = Reminder::new(
            uid,
            "second".into(),
            dayhelper_domain::Recurrence::Once {
                at: Utc.with_ymd_and_hms(2026, 6, 2, 12, 0, 0).unwrap(),
            },
            Utc::now(),
        );
        repo.save(&r1).await.unwrap();
        repo.save(&r2).await.unwrap();

        let uc = ListReminders::new(repo);
        let result = uc.execute(uid).await.unwrap();
        assert_eq!(result.len(), 2);
    }
}

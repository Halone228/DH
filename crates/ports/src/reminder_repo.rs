use async_trait::async_trait;
use dayhelper_domain::{Reminder, ReminderId, UserId};

use crate::errors::RepoError;

#[async_trait]
pub trait ReminderRepo: Send + Sync {
    async fn save(&self, reminder: &Reminder) -> Result<(), RepoError>;
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Reminder>, RepoError>;
    async fn get(&self, id: ReminderId) -> Result<Option<Reminder>, RepoError>;
    async fn deactivate(&self, id: ReminderId) -> Result<(), RepoError>;
}

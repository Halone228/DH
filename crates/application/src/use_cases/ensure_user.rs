use std::sync::Arc;

use chrono_tz::Tz;
use dayhelper_domain::{TelegramUserId, User};
use dayhelper_ports::UserRepo;

use crate::AppError;

/// Idempotent registration. Called on every `/start` and on first contact
/// from the TMA so the rest of the system always has a `User` to work with.
pub struct EnsureUser {
    users: Arc<dyn UserRepo>,
}

impl EnsureUser {
    pub fn new(users: Arc<dyn UserRepo>) -> Self {
        Self { users }
    }

    pub async fn execute(
        &self,
        telegram_id: TelegramUserId,
        timezone: Tz,
    ) -> Result<User, AppError> {
        if let Some(existing) = self.users.find_by_telegram_id(telegram_id).await? {
            return Ok(existing);
        }
        let user = User::new(telegram_id, timezone);
        self.users.upsert(&user).await?;
        Ok(user)
    }
}

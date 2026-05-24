use std::sync::Arc;

use chrono_tz::Tz;
use dayhelper_domain::{TelegramUserId, User};
use dayhelper_ports::UserRepo;

use crate::AppError;

/// Result of [`EnsureUser::execute`] — distinguishes first-time users from
/// returning ones so callers can tailor onboarding messages.
#[derive(Debug)]
pub enum EnsureResult {
    New(User),
    Existing(User),
}

impl EnsureResult {
    pub fn user(&self) -> &User {
        match self {
            EnsureResult::New(u) | EnsureResult::Existing(u) => u,
        }
    }
}

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
    ) -> Result<EnsureResult, AppError> {
        if let Some(existing) = self.users.find_by_telegram_id(telegram_id).await? {
            return Ok(EnsureResult::Existing(existing));
        }
        let user = User::new(telegram_id, timezone);
        self.users.upsert(&user).await?;
        Ok(EnsureResult::New(user))
    }
}

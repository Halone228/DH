use std::sync::Arc;

use dayhelper_domain::ids::UserId;
use dayhelper_ports::UserRepo;

use crate::AppError;

/// Change the stored timezone for an existing user.
pub struct UpdateTimezone {
    users: Arc<dyn UserRepo>,
}

impl UpdateTimezone {
    pub fn new(users: Arc<dyn UserRepo>) -> Self {
        Self { users }
    }

    pub async fn execute(&self, user_id: UserId, timezone: &str) -> Result<(), AppError> {
        let tz: chrono_tz::Tz = timezone
            .parse()
            .map_err(|_| AppError::Invalid("Неверный часовой пояс".into()))?;

        let mut user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::NotFound)?;

        user.timezone = tz;
        self.users.upsert(&user).await?;
        Ok(())
    }
}

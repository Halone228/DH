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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::FakeUserRepo;
    use chrono_tz::Europe::Moscow;
    use dayhelper_domain::TelegramUserId;

    #[tokio::test]
    async fn test_valid_timezone_updates() {
        let repo = Arc::new(FakeUserRepo::new());
        let uc = UpdateTimezone::new(repo.clone());

        let tg = TelegramUserId(1);
        let user = dayhelper_domain::User::new(tg, Moscow);
        let uid = user.id;
        repo.upsert(&user).await.unwrap();

        uc.execute(uid, "Asia/Yekaterinburg").await.unwrap();
        let updated = repo.find_by_id(uid).await.unwrap().unwrap();
        assert_eq!(
            updated.timezone.name(),
            "Asia/Yekaterinburg"
        );
    }

    #[tokio::test]
    async fn test_invalid_timezone_fails() {
        let repo = Arc::new(FakeUserRepo::new());
        let uc = UpdateTimezone::new(repo.clone());

        let tg = TelegramUserId(1);
        let user = dayhelper_domain::User::new(tg, Moscow);
        let uid = user.id;
        repo.upsert(&user).await.unwrap();

        let result = uc.execute(uid, "Invalid/Zone").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::AppError::Invalid(_)));
    }
}

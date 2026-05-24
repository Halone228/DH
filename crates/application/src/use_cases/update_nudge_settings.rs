use std::sync::Arc;

use chrono::NaiveTime;
use dayhelper_domain::nudge::NudgeSettings;
use dayhelper_domain::ids::UserId;
use dayhelper_ports::NudgeSettingsRepo;

use crate::AppError;

/// Read and write nudge preferences for a user.
pub struct UpdateNudgeSettings {
    nudge_settings: Arc<dyn NudgeSettingsRepo>,
}

impl UpdateNudgeSettings {
    pub fn new(nudge_settings: Arc<dyn NudgeSettingsRepo>) -> Self {
        Self { nudge_settings }
    }

    /// Return current settings, creating defaults on first access.
    pub async fn get(&self, user_id: UserId) -> Result<NudgeSettings, AppError> {
        match self.nudge_settings.get(user_id).await? {
            Some(s) => Ok(s),
            None => Ok(NudgeSettings::default_for(user_id)),
        }
    }

    /// Toggle the enabled flag.
    pub async fn set_enabled(&self, user_id: UserId, enabled: bool) -> Result<(), AppError> {
        let mut settings = self.get(user_id).await?;
        settings.enabled = enabled;
        self.nudge_settings.save(&settings).await?;
        Ok(())
    }

    /// Update the active window. Validates that start < end.
    pub async fn set_window(
        &self,
        user_id: UserId,
        start: NaiveTime,
        end: NaiveTime,
    ) -> Result<(), AppError> {
        if start >= end {
            return Err(AppError::Invalid(
                "Начало окна должно быть раньше конца".into(),
            ));
        }
        let mut settings = self.get(user_id).await?;
        settings.active_window_start = start;
        settings.active_window_end = end;
        self.nudge_settings.save(&settings).await?;
        Ok(())
    }

    /// Update the daily nudge count.
    pub async fn set_daily_count(
        &self,
        user_id: UserId,
        count: u8,
    ) -> Result<(), AppError> {
        let mut settings = self.get(user_id).await?;
        settings.daily_count = count;
        self.nudge_settings.save(&settings).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::FakeNudgeSettingsRepo;
    use chrono::NaiveTime;
    use dayhelper_domain::ids::UserId;

    fn make_uc() -> (Arc<FakeNudgeSettingsRepo>, UpdateNudgeSettings) {
        let repo = Arc::new(FakeNudgeSettingsRepo::new());
        let uc = UpdateNudgeSettings::new(repo.clone());
        (repo, uc)
    }

    #[tokio::test]
    async fn test_set_enabled() {
        let (_, uc) = make_uc();
        let uid = UserId::new();
        uc.set_enabled(uid, true).await.unwrap();
        let settings = uc.get(uid).await.unwrap();
        assert!(settings.enabled);
    }

    #[tokio::test]
    async fn test_set_window() {
        let (_, uc) = make_uc();
        let uid = UserId::new();
        let start = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        uc.set_window(uid, start, end).await.unwrap();
        let settings = uc.get(uid).await.unwrap();
        assert_eq!(settings.active_window_start, start);
        assert_eq!(settings.active_window_end, end);
    }

    #[tokio::test]
    async fn test_set_daily_count() {
        let (_, uc) = make_uc();
        let uid = UserId::new();
        uc.set_daily_count(uid, 10).await.unwrap();
        let settings = uc.get(uid).await.unwrap();
        assert_eq!(settings.daily_count, 10);
    }

    #[tokio::test]
    async fn test_invalid_window_start_after_end() {
        let (_, uc) = make_uc();
        let uid = UserId::new();
        let start = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let result = uc.set_window(uid, start, end).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::AppError::Invalid(_)));
    }
}

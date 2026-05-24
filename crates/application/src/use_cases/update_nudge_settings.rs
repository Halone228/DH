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
}

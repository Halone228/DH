use async_trait::async_trait;
use chrono::NaiveTime;
use dayhelper_domain::{NudgeSettings, UserId};
use dayhelper_ports::{NudgeSettingsRepo, RepoError};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteNudgeSettingsRepo {
    pool: SqlitePool,
}

impl SqliteNudgeSettingsRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NudgeSettingsRepo for SqliteNudgeSettingsRepo {
    async fn save(&self, s: &NudgeSettings) -> Result<(), RepoError> {
        sqlx::query(
            r#"
            INSERT INTO nudge_settings (user_id, enabled, daily_count, active_window_start, active_window_end)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(user_id) DO UPDATE SET
                enabled              = excluded.enabled,
                daily_count          = excluded.daily_count,
                active_window_start  = excluded.active_window_start,
                active_window_end    = excluded.active_window_end
            "#,
        )
        .bind(s.user_id.0.to_string())
        .bind(s.enabled as i32)
        .bind(s.daily_count as i32)
        .bind(s.active_window_start.format("%H:%M:%S").to_string())
        .bind(s.active_window_end.format("%H:%M:%S").to_string())
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn get(&self, user_id: UserId) -> Result<Option<NudgeSettings>, RepoError> {
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT user_id, enabled, daily_count, active_window_start, active_window_end
            FROM nudge_settings
            WHERE user_id = ?
            "#,
        )
        .bind(user_id.0.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        row.map(Row::into_settings).transpose()
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    user_id: String,
    enabled: i64,
    daily_count: i64,
    active_window_start: String,
    active_window_end: String,
}

impl Row {
    fn into_settings(self) -> Result<NudgeSettings, RepoError> {
        let id = Uuid::parse_str(&self.user_id).map_err(RepoError::storage)?;
        let start = NaiveTime::parse_from_str(&self.active_window_start, "%H:%M:%S")
            .map_err(RepoError::storage)?;
        let end = NaiveTime::parse_from_str(&self.active_window_end, "%H:%M:%S")
            .map_err(RepoError::storage)?;
        Ok(NudgeSettings {
            user_id: UserId(id),
            enabled: self.enabled != 0,
            daily_count: self.daily_count.clamp(0, u8::MAX as i64) as u8,
            active_window_start: start,
            active_window_end: end,
        })
    }
}

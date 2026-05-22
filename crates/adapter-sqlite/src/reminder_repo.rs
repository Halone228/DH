use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_domain::{Recurrence, Reminder, ReminderId, UserId};
use dayhelper_ports::{ReminderRepo, RepoError};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteReminderRepo {
    pool: SqlitePool,
}

impl SqliteReminderRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReminderRepo for SqliteReminderRepo {
    async fn save(&self, r: &Reminder) -> Result<(), RepoError> {
        let recurrence = serde_json::to_string(&r.recurrence).map_err(RepoError::storage)?;
        sqlx::query(
            r#"
            INSERT INTO reminders (id, user_id, text, recurrence_json, active, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                text            = excluded.text,
                recurrence_json = excluded.recurrence_json,
                active          = excluded.active
            "#,
        )
        .bind(r.id.0.to_string())
        .bind(r.user_id.0.to_string())
        .bind(&r.text)
        .bind(recurrence)
        .bind(r.active as i32)
        .bind(r.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Reminder>, RepoError> {
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, user_id, text, recurrence_json, active, created_at
            FROM reminders
            WHERE user_id = ? AND active = 1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.0.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        rows.into_iter().map(Row::into_domain).collect()
    }

    async fn get(&self, id: ReminderId) -> Result<Option<Reminder>, RepoError> {
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, user_id, text, recurrence_json, active, created_at
            FROM reminders WHERE id = ?
            "#,
        )
        .bind(id.0.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        row.map(Row::into_domain).transpose()
    }

    async fn deactivate(&self, id: ReminderId) -> Result<(), RepoError> {
        sqlx::query("UPDATE reminders SET active = 0 WHERE id = ?")
            .bind(id.0.to_string())
            .execute(&self.pool)
            .await
            .map_err(RepoError::storage)?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: String,
    user_id: String,
    text: String,
    recurrence_json: String,
    active: i64,
    created_at: String,
}

impl Row {
    fn into_domain(self) -> Result<Reminder, RepoError> {
        let id = Uuid::parse_str(&self.id).map_err(RepoError::storage)?;
        let user_id = Uuid::parse_str(&self.user_id).map_err(RepoError::storage)?;
        let recurrence: Recurrence =
            serde_json::from_str(&self.recurrence_json).map_err(RepoError::storage)?;
        let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(RepoError::storage)?
            .with_timezone(&Utc);
        Ok(Reminder {
            id: ReminderId(id),
            user_id: UserId(user_id),
            text: self.text,
            recurrence,
            active: self.active != 0,
            created_at,
        })
    }
}

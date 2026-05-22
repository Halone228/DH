use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_desktop_domain::{LocalNotification, LocalNotificationState};
use dayhelper_desktop_ports::{LocalNotificationRepo, RepoError};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteNotificationRepo {
    pool: SqlitePool,
}

impl SqliteNotificationRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LocalNotificationRepo for SqliteNotificationRepo {
    async fn upsert(&self, n: &LocalNotification) -> Result<(), RepoError> {
        sqlx::query(
            r#"
            INSERT INTO local_notifications (id, title, body, fire_at, category, state, fired_at, ack_pending)
            VALUES (?, ?, ?, ?, ?, ?, ?, 0)
            ON CONFLICT(id) DO UPDATE SET
                title    = excluded.title,
                body     = excluded.body,
                fire_at  = excluded.fire_at,
                category = excluded.category
            -- intentionally do not overwrite state/fired_at on resync; once
            -- fired locally, our row is the source of truth.
            "#,
        )
        .bind(n.id.to_string())
        .bind(&n.title)
        .bind(&n.body)
        .bind(n.fire_at.to_rfc3339())
        .bind(&n.category)
        .bind(state_to_str(n.state))
        .bind(n.fired_at.map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn pending_due(&self, now: DateTime<Utc>) -> Result<Vec<LocalNotification>, RepoError> {
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, title, body, fire_at, category, state, fired_at
            FROM local_notifications
            WHERE state = 'pending' AND fire_at <= ?
            ORDER BY fire_at ASC
            "#,
        )
        .bind(now.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        rows.into_iter().map(Row::into_domain).collect()
    }

    async fn mark(
        &self,
        id: Uuid,
        state: LocalNotificationState,
        fired_at: Option<DateTime<Utc>>,
    ) -> Result<(), RepoError> {
        let ack_pending = matches!(state, LocalNotificationState::Fired);
        sqlx::query(
            r#"
            UPDATE local_notifications
            SET state = ?, fired_at = ?, ack_pending = ?
            WHERE id = ?
            "#,
        )
        .bind(state_to_str(state))
        .bind(fired_at.map(|t| t.to_rfc3339()))
        .bind(ack_pending as i32)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn fired_pending_ack(&self) -> Result<Vec<Uuid>, RepoError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT id FROM local_notifications WHERE ack_pending = 1",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        rows.into_iter()
            .map(|(s,)| Uuid::parse_str(&s).map_err(RepoError::storage))
            .collect()
    }

    async fn clear_fired_acks(&self, ids: &[Uuid]) -> Result<(), RepoError> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await.map_err(RepoError::storage)?;
        for id in ids {
            sqlx::query("UPDATE local_notifications SET ack_pending = 0 WHERE id = ?")
                .bind(id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(RepoError::storage)?;
        }
        tx.commit().await.map_err(RepoError::storage)?;
        Ok(())
    }
}

fn state_to_str(s: LocalNotificationState) -> &'static str {
    match s {
        LocalNotificationState::Pending => "pending",
        LocalNotificationState::Fired => "fired",
        LocalNotificationState::Skipped => "skipped",
    }
}

fn str_to_state(s: &str) -> Result<LocalNotificationState, RepoError> {
    match s {
        "pending" => Ok(LocalNotificationState::Pending),
        "fired" => Ok(LocalNotificationState::Fired),
        "skipped" => Ok(LocalNotificationState::Skipped),
        other => Err(RepoError::Storage(
            format!("unknown notification state {other}").into(),
        )),
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: String,
    title: String,
    body: String,
    fire_at: String,
    category: String,
    state: String,
    fired_at: Option<String>,
}

impl Row {
    fn into_domain(self) -> Result<LocalNotification, RepoError> {
        let id = Uuid::parse_str(&self.id).map_err(RepoError::storage)?;
        let fire_at = DateTime::parse_from_rfc3339(&self.fire_at)
            .map_err(RepoError::storage)?
            .with_timezone(&Utc);
        let fired_at = self
            .fired_at
            .map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|d| d.with_timezone(&Utc))
                    .map_err(RepoError::storage)
            })
            .transpose()?;
        Ok(LocalNotification {
            id,
            title: self.title,
            body: self.body,
            fire_at,
            category: self.category,
            state: str_to_state(&self.state)?,
            fired_at,
        })
    }
}

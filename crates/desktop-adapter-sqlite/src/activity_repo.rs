use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_desktop_domain::{ActivityEvent, ActivityEventId};
use dayhelper_desktop_ports::{LocalActivityRepo, RepoError};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteActivityRepo {
    pool: SqlitePool,
}

impl SqliteActivityRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LocalActivityRepo for SqliteActivityRepo {
    async fn append(&self, e: &ActivityEvent) -> Result<(), RepoError> {
        sqlx::query(
            r#"
            INSERT INTO activity_events (id, app_name, window_title, started_at, ended_at, synced)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(e.id.0.to_string())
        .bind(&e.app_name)
        .bind(e.window_title.as_deref())
        .bind(e.started_at.to_rfc3339())
        .bind(e.ended_at.to_rfc3339())
        .bind(e.synced as i32)
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn unsynced(&self, limit: u32) -> Result<Vec<ActivityEvent>, RepoError> {
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, app_name, window_title, started_at, ended_at, synced
            FROM activity_events
            WHERE synced = 0
            ORDER BY ended_at ASC
            LIMIT ?
            "#,
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        rows.into_iter().map(Row::into_domain).collect()
    }

    async fn mark_synced(&self, ids: &[Uuid]) -> Result<(), RepoError> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await.map_err(RepoError::storage)?;
        for id in ids {
            sqlx::query("UPDATE activity_events SET synced = 1 WHERE id = ?")
                .bind(id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(RepoError::storage)?;
        }
        tx.commit().await.map_err(RepoError::storage)?;
        Ok(())
    }

    async fn prune_synced_before(&self, threshold: DateTime<Utc>) -> Result<u64, RepoError> {
        let res = sqlx::query(
            "DELETE FROM activity_events WHERE synced = 1 AND ended_at < ?",
        )
        .bind(threshold.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(res.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: String,
    app_name: String,
    window_title: Option<String>,
    started_at: String,
    ended_at: String,
    synced: i64,
}

impl Row {
    fn into_domain(self) -> Result<ActivityEvent, RepoError> {
        let id = Uuid::parse_str(&self.id).map_err(RepoError::storage)?;
        let started_at = DateTime::parse_from_rfc3339(&self.started_at)
            .map_err(RepoError::storage)?
            .with_timezone(&Utc);
        let ended_at = DateTime::parse_from_rfc3339(&self.ended_at)
            .map_err(RepoError::storage)?
            .with_timezone(&Utc);
        Ok(ActivityEvent {
            id: ActivityEventId(id),
            app_name: self.app_name,
            window_title: self.window_title,
            started_at,
            ended_at,
            synced: self.synced != 0,
        })
    }
}

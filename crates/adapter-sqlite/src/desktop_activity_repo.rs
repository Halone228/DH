use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_ports::{DesktopActivityRepo, DesktopActivityRow, RepoError};
use sqlx::SqlitePool;

pub struct SqliteDesktopActivityRepo {
    pool: SqlitePool,
}

impl SqliteDesktopActivityRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DesktopActivityRepo for SqliteDesktopActivityRepo {
    async fn append_batch(&self, rows: &[DesktopActivityRow]) -> Result<(), RepoError> {
        if rows.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await.map_err(RepoError::storage)?;
        for r in rows {
            sqlx::query(
                r#"
                INSERT INTO desktop_activity
                    (id, user_id, app_name, window_title, started_at, ended_at, received_at)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(r.id.to_string())
            .bind(r.user_id.0.to_string())
            .bind(&r.app_name)
            .bind(r.window_title.as_deref())
            .bind(r.started_at.to_rfc3339())
            .bind(r.ended_at.to_rfc3339())
            .bind(r.received_at.to_rfc3339())
            .execute(&mut *tx)
            .await
            .map_err(RepoError::storage)?;
        }
        tx.commit().await.map_err(RepoError::storage)?;
        Ok(())
    }

    async fn prune_before(&self, threshold: DateTime<Utc>) -> Result<u64, RepoError> {
        let res = sqlx::query("DELETE FROM desktop_activity WHERE received_at < ?")
            .bind(threshold.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(RepoError::storage)?;
        Ok(res.rows_affected())
    }
}

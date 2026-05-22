use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_domain::{JobId, ReminderId, UserId};
use dayhelper_ports::{JobKind, JobQueue, RepoError, ScheduledJob};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteJobQueue {
    pool: SqlitePool,
}

impl SqliteJobQueue {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobQueue for SqliteJobQueue {
    async fn enqueue(&self, job: ScheduledJob) -> Result<(), RepoError> {
        let (kind, reminder_id, payload) = match &job.kind {
            JobKind::Reminder { reminder_id } => {
                ("reminder", Some(reminder_id.0.to_string()), None)
            }
            JobKind::Nudge { message } => ("nudge", None, Some(message.clone())),
        };

        sqlx::query(
            r#"
            INSERT INTO scheduled_jobs (id, user_id, kind, reminder_id, payload, fire_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(job.id.0.to_string())
        .bind(job.user_id.0.to_string())
        .bind(kind)
        .bind(reminder_id)
        .bind(payload)
        .bind(job.fire_at.to_rfc3339())
        .bind(job.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn pop_due(&self, now: DateTime<Utc>) -> Result<Option<ScheduledJob>, RepoError> {
        // Atomically claim one due row by stamping `fired_at` and returning it.
        // SQLite's `RETURNING` is supported since 3.35.
        let row: Option<JobRow> = sqlx::query_as(
            r#"
            UPDATE scheduled_jobs
            SET fired_at = ?
            WHERE id = (
                SELECT id FROM scheduled_jobs
                WHERE fired_at IS NULL AND fire_at <= ?
                ORDER BY fire_at ASC
                LIMIT 1
            )
            RETURNING id, user_id, kind, reminder_id, payload, fire_at, created_at
            "#,
        )
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .fetch_optional(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        row.map(JobRow::into_domain).transpose()
    }

    async fn peek_next_fire_at(&self) -> Result<Option<DateTime<Utc>>, RepoError> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT fire_at FROM scheduled_jobs
            WHERE fired_at IS NULL
            ORDER BY fire_at ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        row.map(|(s,)| {
            DateTime::parse_from_rfc3339(&s)
                .map_err(RepoError::storage)
                .map(|dt| dt.with_timezone(&Utc))
        })
        .transpose()
    }

    async fn pending_for_user_until(
        &self,
        user_id: UserId,
        until: DateTime<Utc>,
    ) -> Result<Vec<ScheduledJob>, RepoError> {
        let rows: Vec<JobRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, kind, reminder_id, payload, fire_at, created_at
            FROM scheduled_jobs
            WHERE user_id = ? AND fired_at IS NULL AND fire_at <= ?
            ORDER BY fire_at ASC
            "#,
        )
        .bind(user_id.0.to_string())
        .bind(until.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        rows.into_iter().map(JobRow::into_domain).collect()
    }

    async fn count_pending_nudges_in_window(
        &self,
        user_id: UserId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<u64, RepoError> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM scheduled_jobs
            WHERE user_id = ?
              AND kind = 'nudge'
              AND fired_at IS NULL
              AND fire_at >= ?
              AND fire_at < ?
            "#,
        )
        .bind(user_id.0.to_string())
        .bind(start.to_rfc3339())
        .bind(end.to_rfc3339())
        .fetch_one(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(row.0.max(0) as u64)
    }

    async fn prune_fired_before(&self, threshold: DateTime<Utc>) -> Result<u64, RepoError> {
        let res = sqlx::query(
            "DELETE FROM scheduled_jobs WHERE fired_at IS NOT NULL AND fired_at < ?",
        )
        .bind(threshold.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(res.rows_affected())
    }

    async fn cancel_for_reminder(&self, reminder_id: ReminderId) -> Result<(), RepoError> {
        sqlx::query(
            "DELETE FROM scheduled_jobs WHERE reminder_id = ? AND fired_at IS NULL",
        )
        .bind(reminder_id.0.to_string())
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn cancel_nudges_for_user(&self, user_id: UserId) -> Result<(), RepoError> {
        sqlx::query(
            "DELETE FROM scheduled_jobs WHERE user_id = ? AND kind = 'nudge' AND fired_at IS NULL",
        )
        .bind(user_id.0.to_string())
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct JobRow {
    id: String,
    user_id: String,
    kind: String,
    reminder_id: Option<String>,
    payload: Option<String>,
    fire_at: String,
    created_at: String,
}

impl JobRow {
    fn into_domain(self) -> Result<ScheduledJob, RepoError> {
        let id = Uuid::parse_str(&self.id).map_err(RepoError::storage)?;
        let user_id = Uuid::parse_str(&self.user_id).map_err(RepoError::storage)?;
        let fire_at = DateTime::parse_from_rfc3339(&self.fire_at)
            .map_err(RepoError::storage)?
            .with_timezone(&Utc);
        let created_at = DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(RepoError::storage)?
            .with_timezone(&Utc);

        let kind = match self.kind.as_str() {
            "reminder" => {
                let rid = self
                    .reminder_id
                    .ok_or_else(|| RepoError::Storage("reminder job missing reminder_id".into()))?;
                JobKind::Reminder {
                    reminder_id: ReminderId(
                        Uuid::parse_str(&rid).map_err(RepoError::storage)?,
                    ),
                }
            }
            "nudge" => JobKind::Nudge {
                message: self
                    .payload
                    .ok_or_else(|| RepoError::Storage("nudge job missing payload".into()))?,
            },
            other => {
                return Err(RepoError::Storage(
                    format!("unknown job kind: {other}").into(),
                ))
            }
        };

        Ok(ScheduledJob {
            id: JobId(id),
            user_id: UserId(user_id),
            kind,
            fire_at,
            created_at,
        })
    }
}

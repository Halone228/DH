use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_domain::{DesktopToken, DesktopTokenId, UserId};
use dayhelper_ports::{DesktopTokenRepo, RepoError};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteDesktopTokenRepo {
    pool: SqlitePool,
}

impl SqliteDesktopTokenRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DesktopTokenRepo for SqliteDesktopTokenRepo {
    async fn insert(&self, t: &DesktopToken) -> Result<(), RepoError> {
        sqlx::query(
            r#"
            INSERT INTO desktop_tokens (id, user_id, token_hash, label, created_at, last_seen_at, revoked_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(t.id.0.to_string())
        .bind(t.user_id.0.to_string())
        .bind(&t.token_hash)
        .bind(&t.label)
        .bind(t.created_at.to_rfc3339())
        .bind(t.last_seen_at.map(|d| d.to_rfc3339()))
        .bind(t.revoked_at.map(|d| d.to_rfc3339()))
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn find_active_by_hash(&self, hash: &str) -> Result<Option<DesktopToken>, RepoError> {
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, user_id, token_hash, label, created_at, last_seen_at, revoked_at
            FROM desktop_tokens
            WHERE token_hash = ? AND revoked_at IS NULL
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        row.map(Row::into_domain).transpose()
    }

    async fn touch_last_seen(
        &self,
        id: DesktopTokenId,
        at: DateTime<Utc>,
    ) -> Result<(), RepoError> {
        sqlx::query("UPDATE desktop_tokens SET last_seen_at = ? WHERE id = ?")
            .bind(at.to_rfc3339())
            .bind(id.0.to_string())
            .execute(&self.pool)
            .await
            .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn revoke(&self, id: DesktopTokenId) -> Result<(), RepoError> {
        sqlx::query("UPDATE desktop_tokens SET revoked_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(id.0.to_string())
            .execute(&self.pool)
            .await
            .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn list_active_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<DesktopToken>, RepoError> {
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, user_id, token_hash, label, created_at, last_seen_at, revoked_at
            FROM desktop_tokens
            WHERE user_id = ? AND revoked_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.0.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        rows.into_iter().map(Row::into_domain).collect()
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: String,
    user_id: String,
    token_hash: String,
    label: String,
    created_at: String,
    last_seen_at: Option<String>,
    revoked_at: Option<String>,
}

impl Row {
    fn into_domain(self) -> Result<DesktopToken, RepoError> {
        let id = Uuid::parse_str(&self.id).map_err(RepoError::storage)?;
        let user_id = Uuid::parse_str(&self.user_id).map_err(RepoError::storage)?;
        let created_at = DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(RepoError::storage)?
            .with_timezone(&Utc);
        let last_seen_at = self
            .last_seen_at
            .map(|s| DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&Utc)))
            .transpose()
            .map_err(RepoError::storage)?;
        let revoked_at = self
            .revoked_at
            .map(|s| DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&Utc)))
            .transpose()
            .map_err(RepoError::storage)?;
        Ok(DesktopToken {
            id: DesktopTokenId(id),
            user_id: UserId(user_id),
            token_hash: self.token_hash,
            label: self.label,
            created_at,
            last_seen_at,
            revoked_at,
        })
    }
}

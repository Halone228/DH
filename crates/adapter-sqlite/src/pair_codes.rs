use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use dayhelper_domain::UserId;
use dayhelper_ports::{PairCodeStore, RepoError};
use rand::Rng;
use sqlx::SqlitePool;
use uuid::Uuid;

/// SQLite-backed pair-code store. Survives process restarts.
/// Expired codes are pruned on every `issue()` call.
pub struct SqlitePairCodeStore {
    pool: SqlitePool,
}

#[derive(sqlx::FromRow)]
struct PairCodeRow {
    user_id: String,
}

impl SqlitePairCodeStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PairCodeStore for SqlitePairCodeStore {
    async fn issue(
        &self,
        user_id: UserId,
        ttl: Duration,
        now: DateTime<Utc>,
    ) -> Result<String, RepoError> {
        // Generate 6-digit code with leading zeros preserved.
        // ThreadRng is !Send, so it must be dropped before any .await.
        let code = {
            let mut rng = rand::thread_rng();
            let n: u32 = rng.gen_range(0..1_000_000);
            format!("{n:06}")
        };
        let expires_at = (now + ttl).to_rfc3339();

        // Prune expired codes.
        sqlx::query("DELETE FROM pair_codes WHERE expires_at < ?")
            .bind(now.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(RepoError::storage)?;

        // On collision with a live code, the old one is invalidated (same
        // behaviour as the in-memory adapter).
        sqlx::query(
            "INSERT OR REPLACE INTO pair_codes (code, user_id, expires_at) VALUES (?, ?, ?)",
        )
        .bind(&code)
        .bind(user_id.0.to_string())
        .bind(&expires_at)
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;

        Ok(code)
    }

    async fn redeem(
        &self,
        code: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<UserId>, RepoError> {
        // Fetch then delete — atomic enough for single-process SQLite
        // (single writer, WAL mode). If two concurrent requests race,
        // one gets Some, the other gets None — same as the in-memory impl.
        let row: Option<PairCodeRow> = sqlx::query_as(
            "SELECT user_id FROM pair_codes WHERE code = ? AND expires_at > ?",
        )
        .bind(code)
        .bind(now.to_rfc3339())
        .fetch_optional(&self.pool)
        .await
        .map_err(RepoError::storage)?;

        let Some(row) = row else {
            return Ok(None);
        };

        sqlx::query("DELETE FROM pair_codes WHERE code = ?")
            .bind(code)
            .execute(&self.pool)
            .await
            .map_err(RepoError::storage)?;

        let id = Uuid::parse_str(&row.user_id).map_err(RepoError::storage)?;
        Ok(Some(UserId(id)))
    }
}

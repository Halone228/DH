use async_trait::async_trait;
use chrono_tz::Tz;
use dayhelper_domain::{TelegramUserId, User, UserId};
use dayhelper_ports::{RepoError, UserRepo};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteUserRepo {
    pool: SqlitePool,
}

impl SqliteUserRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepo for SqliteUserRepo {
    async fn upsert(&self, user: &User) -> Result<(), RepoError> {
        let id = user.id.0.to_string();
        let tz = user.timezone.name().to_string();
        sqlx::query(
            r#"
            INSERT INTO users (id, telegram_id, username, timezone, locale)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(telegram_id) DO UPDATE SET
                username = excluded.username,
                timezone = excluded.timezone,
                locale   = excluded.locale
            "#,
        )
        .bind(id)
        .bind(user.telegram_id.0)
        .bind(user.username.as_deref())
        .bind(tz)
        .bind(&user.locale)
        .execute(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepoError> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"SELECT id, telegram_id, username, timezone, locale FROM users WHERE id = ?"#,
        )
        .bind(id.0.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        row.map(UserRow::into_user).transpose()
    }

    async fn find_by_telegram_id(
        &self,
        telegram_id: TelegramUserId,
    ) -> Result<Option<User>, RepoError> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"SELECT id, telegram_id, username, timezone, locale FROM users WHERE telegram_id = ?"#,
        )
        .bind(telegram_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        row.map(UserRow::into_user).transpose()
    }

    async fn list_with_nudges_enabled(&self) -> Result<Vec<User>, RepoError> {
        let rows: Vec<UserRow> = sqlx::query_as(
            r#"
            SELECT u.id, u.telegram_id, u.username, u.timezone, u.locale
            FROM users u
            LEFT JOIN nudge_settings ns ON ns.user_id = u.id
            WHERE COALESCE(ns.enabled, 1) = 1
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepoError::storage)?;
        rows.into_iter().map(UserRow::into_user).collect()
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    telegram_id: i64,
    username: Option<String>,
    timezone: String,
    locale: String,
}

impl UserRow {
    fn into_user(self) -> Result<User, RepoError> {
        let id = Uuid::parse_str(&self.id).map_err(RepoError::storage)?;
        let tz: Tz = self.timezone.parse().map_err(|_| {
            RepoError::Storage(format!("invalid timezone: {}", self.timezone).into())
        })?;
        Ok(User {
            id: UserId(id),
            telegram_id: TelegramUserId(self.telegram_id),
            username: self.username,
            timezone: tz,
            locale: self.locale,
        })
    }
}

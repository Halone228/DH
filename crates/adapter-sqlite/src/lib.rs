//! SQLite implementations of the persistent ports.
//!
//! Migrations live in `crates/adapter-sqlite/migrations/`. Run them via
//! `Pool::migrate`, exposed by [`migrate`].

pub mod backup;
mod desktop_activity_repo;
mod desktop_token_repo;
mod job_queue;
mod nudge_settings;
mod pair_codes;
mod reminder_repo;
mod user_repo;

pub use sqlx::SqlitePool;

pub use desktop_activity_repo::SqliteDesktopActivityRepo;
pub use desktop_token_repo::SqliteDesktopTokenRepo;
pub use job_queue::SqliteJobQueue;
pub use nudge_settings::SqliteNudgeSettingsRepo;
pub use pair_codes::SqlitePairCodeStore;
pub use reminder_repo::SqliteReminderRepo;
pub use user_repo::SqliteUserRepo;

/// Apply all migrations bundled with this crate. Call once at startup.
pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Connect to a SQLite URL with sensible defaults (creates the DB if missing,
/// enables WAL, foreign keys, busy timeout).
pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;

    let opts = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_secs(5));

    SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await
}

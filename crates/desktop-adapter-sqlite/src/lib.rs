//! Local SQLite store for the desktop client.
//!
//! Two responsibilities:
//!  1. buffer activity events + pending notifications (sqlite tables);
//!  2. persist credentials to a TOML file with mode 600 (`FileCredentialsStore`).

mod activity_repo;
mod credentials;
mod notification_repo;

pub use activity_repo::SqliteActivityRepo;
pub use credentials::FileCredentialsStore;
pub use notification_repo::SqliteNotificationRepo;
pub use sqlx::SqlitePool;

pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;

    let opts = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_secs(5));

    SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(opts)
        .await
}

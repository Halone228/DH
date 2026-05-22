use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dayhelper_domain::UserId;

use crate::errors::RepoError;

/// Short-lived pairing codes. A typical implementation is in-memory with a
/// TTL background sweep — codes are tiny and lose no real value if a
/// process restart wipes them (the user just runs `/pair` again).
#[async_trait]
pub trait PairCodeStore: Send + Sync {
    /// Generate a new code, bind it to `user_id`, expire after `ttl`.
    /// Returns the human-typeable code (e.g. a 6-digit string).
    async fn issue(
        &self,
        user_id: UserId,
        ttl: chrono::Duration,
        now: DateTime<Utc>,
    ) -> Result<String, RepoError>;

    /// Consume a code if it matches and isn't expired. Returns the bound
    /// user. Each code is single-use.
    async fn redeem(
        &self,
        code: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<UserId>, RepoError>;
}

use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use dayhelper_domain::UserId;
use dayhelper_ports::{PairCodeStore, RepoError};
use rand::Rng;

/// In-memory single-process implementation. Single-server deployment is the
/// only one we support today; if multi-server arrives, swap with a
/// Redis-backed adapter.
#[derive(Default)]
pub struct MemoryPairCodeStore {
    inner: RwLock<HashMap<String, Entry>>,
}

struct Entry {
    user_id: UserId,
    expires_at: DateTime<Utc>,
}

impl MemoryPairCodeStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn purge_expired(&self, now: DateTime<Utc>) {
        if let Ok(mut guard) = self.inner.write() {
            guard.retain(|_, e| e.expires_at > now);
        }
    }
}

#[async_trait]
impl PairCodeStore for MemoryPairCodeStore {
    async fn issue(
        &self,
        user_id: UserId,
        ttl: Duration,
        now: DateTime<Utc>,
    ) -> Result<String, RepoError> {
        self.purge_expired(now);
        let mut rng = rand::thread_rng();
        // 6-digit code, leading zeros preserved.
        let n: u32 = rng.gen_range(0..1_000_000);
        let code = format!("{n:06}");

        let mut guard = self
            .inner
            .write()
            .map_err(|_| RepoError::Storage("pair-code lock poisoned".into()))?;
        // In the wildly unlikely case of a collision while a previous code is
        // still alive, just overwrite — old code becomes invalid.
        guard.insert(
            code.clone(),
            Entry {
                user_id,
                expires_at: now + ttl,
            },
        );
        Ok(code)
    }

    async fn redeem(
        &self,
        code: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<UserId>, RepoError> {
        self.purge_expired(now);
        let mut guard = self
            .inner
            .write()
            .map_err(|_| RepoError::Storage("pair-code lock poisoned".into()))?;
        match guard.remove(code) {
            Some(e) if e.expires_at > now => Ok(Some(e.user_id)),
            _ => Ok(None),
        }
    }
}

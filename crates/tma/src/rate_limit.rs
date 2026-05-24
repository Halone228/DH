//! Simple in-memory, per-key rate limiter.
//!
//! Keys are typically `telegram_id` (i64) for TMA endpoints or a global
//! sentinel for unauthenticated endpoints like `/api/desktop/pair`.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    requests: Mutex<HashMap<i64, (Instant, u32)>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            max_requests,
            window,
        }
    }

    /// Returns `true` if the request is allowed, `false` if the rate limit
    /// has been exceeded.
    pub async fn check(&self, key: i64) -> bool {
        let mut map = self.requests.lock().await;
        let now = Instant::now();
        let entry = map.entry(key).or_insert((now, 0));
        if now.duration_since(entry.0) > self.window {
            *entry = (now, 1);
            return true;
        }
        if entry.1 < self.max_requests {
            entry.1 += 1;
            true
        } else {
            false
        }
    }

    /// Remove expired entries to prevent unbounded memory growth.
    pub async fn cleanup(&self) {
        let mut map = self.requests.lock().await;
        let now = Instant::now();
        map.retain(|_, (start, _)| now.duration_since(*start) <= self.window);
    }
}

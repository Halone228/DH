use chrono::{DateTime, Utc};

/// Time source. Lifting `now()` behind a port lets tests freeze time and lets
/// us swap in a deterministic clock during scheduler verification.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

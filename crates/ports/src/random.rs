use chrono::{DateTime, Utc};

/// Source of randomness, abstracted so nudge-scheduling tests can be made
/// deterministic by injecting a fake.
pub trait RandomSource: Send + Sync {
    /// Pick `count` distinct timestamps uniformly at random in
    /// `[start, end)`, returned in ascending order.
    fn distinct_in_window(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        count: usize,
    ) -> Vec<DateTime<Utc>>;
}

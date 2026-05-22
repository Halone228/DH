use chrono::{DateTime, Utc};
use dayhelper_ports::RandomSource;
use rand::Rng;

#[derive(Debug, Default, Clone, Copy)]
pub struct OsRandom;

impl RandomSource for OsRandom {
    fn distinct_in_window(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        count: usize,
    ) -> Vec<DateTime<Utc>> {
        if count == 0 || end <= start {
            return Vec::new();
        }

        let span_seconds = (end - start).num_seconds().max(1);
        let mut rng = rand::thread_rng();

        // For small counts vs. large windows we just sample with rejection
        // until we have `count` distinct seconds. The window is typically
        // ~12h ≈ 43200s, count ≤ 5 — collisions are vanishingly rare.
        let mut chosen: Vec<i64> = Vec::with_capacity(count);
        let cap = count.min(span_seconds as usize);
        while chosen.len() < cap {
            let candidate: i64 = rng.gen_range(0..span_seconds);
            if !chosen.contains(&candidate) {
                chosen.push(candidate);
            }
        }

        chosen.sort_unstable();
        chosen
            .into_iter()
            .map(|s| start + chrono::Duration::seconds(s))
            .collect()
    }
}

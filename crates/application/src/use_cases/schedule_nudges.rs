use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use dayhelper_domain::{JobId, NudgeSettings, UserId};
use dayhelper_ports::{Clock, JobKind, JobQueue, RandomSource, ScheduledJob};
use tracing::debug;

use crate::messages::nudge_text;
use crate::AppError;

/// Plans a single user's anti-procrastination nudges for one calendar day
/// (in the user's local timezone). Idempotent per (user, day) only at the
/// caller level — caller should clear existing nudges first or call this
/// at most once per user per day.
pub struct ScheduleDailyNudges {
    jobs: Arc<dyn JobQueue>,
    clock: Arc<dyn Clock>,
    rng: Arc<dyn RandomSource>,
}

impl ScheduleDailyNudges {
    pub fn new(
        jobs: Arc<dyn JobQueue>,
        clock: Arc<dyn Clock>,
        rng: Arc<dyn RandomSource>,
    ) -> Self {
        Self { jobs, clock, rng }
    }

    pub async fn execute(
        &self,
        user_id: UserId,
        timezone: Tz,
        settings: &NudgeSettings,
    ) -> Result<(), AppError> {
        if !settings.enabled || settings.daily_count == 0 {
            return Ok(());
        }

        let now = self.clock.now();
        let (window_start, window_end) =
            todays_window_utc(now, timezone, settings).ok_or_else(|| {
                AppError::Invalid(
                    "could not compute today's nudge window — DST edge?".into(),
                )
            })?;

        // If the window has already passed entirely, schedule for tomorrow.
        let (start, end) = if now >= window_end {
            tomorrows_window_utc(now, timezone, settings).ok_or_else(|| {
                AppError::Invalid("could not compute tomorrow's nudge window".into())
            })?
        } else {
            (now.max(window_start), window_end)
        };

        // Idempotency: if any nudge is already pending in this window for this
        // user, treat the day as planned and skip. This means a second call in
        // the same day is a no-op (which is what the hourly planner relies on).
        let existing = self
            .jobs
            .count_pending_nudges_in_window(user_id, start, end)
            .await?;
        if existing > 0 {
            debug!(user = ?user_id, existing, "nudges already planned, skipping");
            return Ok(());
        }

        let picks = self
            .rng
            .distinct_in_window(start, end, settings.daily_count as usize);

        debug!(user = ?user_id, count = picks.len(), "scheduling nudges");

        for fire_at in picks {
            let job = ScheduledJob {
                id: JobId::new(),
                user_id,
                kind: JobKind::Nudge {
                    message: nudge_text(rand_seed(fire_at)).to_string(),
                },
                fire_at,
                created_at: now,
            };
            self.jobs.enqueue(job).await?;
        }
        Ok(())
    }
}

fn todays_window_utc(
    now: DateTime<Utc>,
    tz: Tz,
    s: &NudgeSettings,
) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let today = now.with_timezone(&tz).date_naive();
    let start = tz
        .from_local_datetime(&today.and_time(s.active_window_start))
        .single()?
        .with_timezone(&Utc);
    let end = tz
        .from_local_datetime(&today.and_time(s.active_window_end))
        .single()?
        .with_timezone(&Utc);
    Some((start, end))
}

fn tomorrows_window_utc(
    now: DateTime<Utc>,
    tz: Tz,
    s: &NudgeSettings,
) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let tomorrow = now.with_timezone(&tz).date_naive().succ_opt()?;
    let start = tz
        .from_local_datetime(&tomorrow.and_time(s.active_window_start))
        .single()?
        .with_timezone(&Utc);
    let end = tz
        .from_local_datetime(&tomorrow.and_time(s.active_window_end))
        .single()?
        .with_timezone(&Utc);
    Some((start, end))
}

fn rand_seed(at: DateTime<Utc>) -> u64 {
    at.timestamp_nanos_opt().unwrap_or(0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{FakeClock, FakeJobQueue, FakeRandomSource};
    use chrono::{TimeZone, Utc};
    use chrono_tz::Europe::Moscow;
    use dayhelper_domain::NudgeSettings;

    fn fixed_now() -> DateTime<Utc> {
        // 10:00 Moscow time — well within the default window
        Moscow
            .with_ymd_and_hms(2026, 5, 24, 10, 0, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn default_settings(uid: UserId) -> NudgeSettings {
        NudgeSettings::default_for(uid)
    }

    #[tokio::test]
    async fn test_schedules_nudges_for_enabled_user() {
        let clock = Arc::new(FakeClock::new(fixed_now()));
        let jobs = Arc::new(FakeJobQueue::new());
        let rng = Arc::new(FakeRandomSource);
        let uc = ScheduleDailyNudges::new(jobs.clone(), clock, rng);

        let uid = UserId::new();
        let settings = default_settings(uid);
        uc.execute(uid, Moscow, &settings).await.unwrap();

        assert_eq!(jobs.len().await, 5);
    }

    #[tokio::test]
    async fn test_skips_disabled_user() {
        let clock = Arc::new(FakeClock::new(fixed_now()));
        let jobs = Arc::new(FakeJobQueue::new());
        let rng = Arc::new(FakeRandomSource);
        let uc = ScheduleDailyNudges::new(jobs.clone(), clock, rng);

        let uid = UserId::new();
        let mut settings = default_settings(uid);
        settings.enabled = false;
        uc.execute(uid, Moscow, &settings).await.unwrap();

        assert_eq!(jobs.len().await, 0);
    }

    #[tokio::test]
    async fn test_idempotent_no_duplicate_nudges() {
        let clock = Arc::new(FakeClock::new(fixed_now()));
        let jobs = Arc::new(FakeJobQueue::new());
        let rng = Arc::new(FakeRandomSource);
        let uc = ScheduleDailyNudges::new(jobs.clone(), clock.clone(), rng);

        let uid = UserId::new();
        let settings = default_settings(uid);
        uc.execute(uid, Moscow, &settings).await.unwrap();
        assert_eq!(jobs.len().await, 5);

        // Second call should be a no-op
        uc.execute(uid, Moscow, &settings).await.unwrap();
        assert_eq!(jobs.len().await, 5, "should not double-schedule");
    }
}

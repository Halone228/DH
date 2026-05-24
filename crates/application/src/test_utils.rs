//! In-memory fake implementations of port traits for testing.
//!
//! Each fake uses `Arc<Mutex<>>` for interior mutability and provides
//! helpers for test assertions.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use dayhelper_domain::{
    DesktopToken, DesktopTokenId, NudgeSettings, Reminder, ReminderId,
    TelegramUserId, User, UserId,
};
use dayhelper_ports::{
    Clock, DesktopActivityRepo, DesktopActivityRow, DesktopTokenRepo, JobKind, JobQueue,
    NotifyError, Notifier, PairCodeStore, RandomSource, ReminderRepo, RepoError,
};
use tokio::sync::Mutex;

// ─── FakeClock ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct FakeClock {
    time: std::sync::Mutex<DateTime<Utc>>,
}

impl FakeClock {
    pub fn new(time: DateTime<Utc>) -> Self {
        Self {
            time: std::sync::Mutex::new(time),
        }
    }

    pub fn set(&self, time: DateTime<Utc>) {
        *self.time.lock().unwrap() = time;
    }

    pub fn advance(&self, d: Duration) {
        let mut t = self.time.lock().unwrap();
        *t += d;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        *self.time.lock().unwrap()
    }
}

// ─── FakeUserRepo ─────────────────────────────────────────────────────────

pub struct FakeUserRepo {
    by_id: Arc<Mutex<HashMap<UserId, User>>>,
    by_tg: Arc<Mutex<HashMap<TelegramUserId, UserId>>>,
}

impl Default for FakeUserRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeUserRepo {
    pub fn new() -> Self {
        Self {
            by_id: Arc::new(Mutex::new(HashMap::new())),
            by_tg: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl dayhelper_ports::UserRepo for FakeUserRepo {
    async fn upsert(&self, user: &User) -> Result<(), RepoError> {
        let mut by_id = self.by_id.lock().await;
        let mut by_tg = self.by_tg.lock().await;
        by_id.insert(user.id, user.clone());
        by_tg.insert(user.telegram_id, user.id);
        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepoError> {
        let by_id = self.by_id.lock().await;
        Ok(by_id.get(&id).cloned())
    }

    async fn find_by_telegram_id(
        &self,
        telegram_id: TelegramUserId,
    ) -> Result<Option<User>, RepoError> {
        let by_id = self.by_id.lock().await;
        let by_tg = self.by_tg.lock().await;
        Ok(by_tg.get(&telegram_id).and_then(|id| by_id.get(id)).cloned())
    }

    async fn list_with_nudges_enabled(&self) -> Result<Vec<User>, RepoError> {
        // This needs NudgeSettingsRepo data; stub out for now
        Ok(vec![])
    }
}

// ─── FakeNudgeSettingsRepo ────────────────────────────────────────────────

pub struct FakeNudgeSettingsRepo {
    settings: Arc<Mutex<HashMap<UserId, NudgeSettings>>>,
}

impl Default for FakeNudgeSettingsRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeNudgeSettingsRepo {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl dayhelper_ports::NudgeSettingsRepo for FakeNudgeSettingsRepo {
    async fn save(&self, s: &NudgeSettings) -> Result<(), RepoError> {
        let mut map = self.settings.lock().await;
        map.insert(s.user_id, s.clone());
        Ok(())
    }

    async fn get(&self, user_id: UserId) -> Result<Option<NudgeSettings>, RepoError> {
        let map = self.settings.lock().await;
        Ok(map.get(&user_id).cloned())
    }
}

// ─── FakeReminderRepo ─────────────────────────────────────────────────────

pub struct FakeReminderRepo {
    reminders: Arc<Mutex<HashMap<ReminderId, Reminder>>>,
}

impl Default for FakeReminderRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeReminderRepo {
    pub fn new() -> Self {
        Self {
            reminders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ReminderRepo for FakeReminderRepo {
    async fn save(&self, reminder: &Reminder) -> Result<(), RepoError> {
        let mut map = self.reminders.lock().await;
        map.insert(reminder.id, reminder.clone());
        Ok(())
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Reminder>, RepoError> {
        let map = self.reminders.lock().await;
        Ok(map
            .values()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn get(&self, id: ReminderId) -> Result<Option<Reminder>, RepoError> {
        let map = self.reminders.lock().await;
        Ok(map.get(&id).cloned())
    }

    async fn deactivate(&self, id: ReminderId) -> Result<(), RepoError> {
        let mut map = self.reminders.lock().await;
        if let Some(r) = map.get_mut(&id) {
            r.active = false;
        }
        Ok(())
    }
}

// ─── FakeJobQueue ─────────────────────────────────────────────────────────

pub struct FakeJobQueue {
    jobs: Arc<Mutex<Vec<dayhelper_ports::ScheduledJob>>>,
}

impl Default for FakeJobQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeJobQueue {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn is_empty(&self) -> bool {
        self.jobs.lock().await.is_empty()
    }

    pub async fn len(&self) -> usize {
        self.jobs.lock().await.len()
    }
}

#[async_trait]
impl JobQueue for FakeJobQueue {
    async fn enqueue(&self, job: dayhelper_ports::ScheduledJob) -> Result<(), RepoError> {
        let mut jobs = self.jobs.lock().await;
        jobs.push(job);
        Ok(())
    }

    async fn pop_due(&self, now: DateTime<Utc>) -> Result<Option<dayhelper_ports::ScheduledJob>, RepoError> {
        let mut jobs = self.jobs.lock().await;
        // Find first due job and remove it (atomic claim)
        if let Some(pos) = jobs.iter().position(|j| j.fire_at <= now) {
            Ok(Some(jobs.remove(pos)))
        } else {
            Ok(None)
        }
    }

    async fn peek_next_fire_at(&self) -> Result<Option<DateTime<Utc>>, RepoError> {
        let jobs = self.jobs.lock().await;
        Ok(jobs.iter().map(|j| j.fire_at).min())
    }

    async fn pending_for_user_until(
        &self,
        user_id: UserId,
        until: DateTime<Utc>,
    ) -> Result<Vec<dayhelper_ports::ScheduledJob>, RepoError> {
        let jobs = self.jobs.lock().await;
        Ok(jobs
            .iter()
            .filter(|j| j.user_id == user_id && j.fire_at <= until)
            .cloned()
            .collect())
    }

    async fn count_pending_nudges_in_window(
        &self,
        user_id: UserId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<u64, RepoError> {
        let jobs = self.jobs.lock().await;
        let count = jobs
            .iter()
            .filter(|j| {
                j.user_id == user_id
                    && j.fire_at >= start
                    && j.fire_at < end
                    && matches!(j.kind, JobKind::Nudge { .. })
            })
            .count();
        Ok(count as u64)
    }

    async fn prune_fired_before(&self, _threshold: DateTime<Utc>) -> Result<u64, RepoError> {
        // Not needed for tests
        Ok(0)
    }

    async fn cancel_for_reminder(&self, reminder_id: ReminderId) -> Result<(), RepoError> {
        let mut jobs = self.jobs.lock().await;
        jobs.retain(|j| !matches!(&j.kind, JobKind::Reminder { reminder_id: rid } if *rid == reminder_id));
        Ok(())
    }

    async fn cancel_nudges_for_user(&self, user_id: UserId) -> Result<(), RepoError> {
        let mut jobs = self.jobs.lock().await;
        jobs.retain(|j| !(j.user_id == user_id && matches!(j.kind, JobKind::Nudge { .. })));
        Ok(())
    }
}

// ─── FakeNotifier ─────────────────────────────────────────────────────────

pub struct FakeNotifier {
    sent: Arc<Mutex<Vec<(TelegramUserId, String)>>>,
}

impl Default for FakeNotifier {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeNotifier {
    pub fn new() -> Self {
        Self {
            sent: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn sent(&self) -> Vec<(TelegramUserId, String)> {
        self.sent.lock().await.clone()
    }
}

#[async_trait]
impl Notifier for FakeNotifier {
    async fn notify(
        &self,
        user: TelegramUserId,
        message: &str,
    ) -> Result<(), NotifyError> {
        self.sent.lock().await.push((user, message.to_string()));
        Ok(())
    }
}

// ─── FakePairCodeStore ────────────────────────────────────────────────────

struct PendingCode {
    user_id: UserId,
    expires_at: DateTime<Utc>,
    used: bool,
}

pub struct FakePairCodeStore {
    codes: Arc<Mutex<HashMap<String, PendingCode>>>,
    counter: Arc<Mutex<u32>>,
}

impl Default for FakePairCodeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FakePairCodeStore {
    pub fn new() -> Self {
        Self {
            codes: Arc::new(Mutex::new(HashMap::new())),
            counter: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl PairCodeStore for FakePairCodeStore {
    async fn issue(
        &self,
        user_id: UserId,
        ttl: Duration,
        now: DateTime<Utc>,
    ) -> Result<String, RepoError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        let code = format!("{:06}", *counter);
        let mut codes = self.codes.lock().await;
        codes.insert(
            code.clone(),
            PendingCode {
                user_id,
                expires_at: now + ttl,
                used: false,
            },
        );
        Ok(code)
    }

    async fn redeem(
        &self,
        code: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<UserId>, RepoError> {
        let mut codes = self.codes.lock().await;
        if let Some(entry) = codes.get_mut(code) {
            if entry.used || now > entry.expires_at {
                return Ok(None);
            }
            entry.used = true;
            Ok(Some(entry.user_id))
        } else {
            Ok(None)
        }
    }
}

// ─── FakeRandomSource ─────────────────────────────────────────────────────

/// Returns evenly-spaced timestamps in the window for deterministic tests.
pub struct FakeRandomSource;

impl RandomSource for FakeRandomSource {
    fn distinct_in_window(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        count: usize,
    ) -> Vec<DateTime<Utc>> {
        if count == 0 {
            return vec![];
        }
        let total_secs = (end - start).num_seconds();
        if total_secs <= 0 || count == 1 {
            return vec![start];
        }
        let step = total_secs / count as i64;
        (0..count)
            .map(|i| start + Duration::seconds(step * i as i64 + step / 2))
            .collect()
    }
}

// ─── FakeDesktopTokenRepo ─────────────────────────────────────────────────

pub struct FakeDesktopTokenRepo {
    tokens: Arc<Mutex<HashMap<DesktopTokenId, DesktopToken>>>,
}

impl Default for FakeDesktopTokenRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeDesktopTokenRepo {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl DesktopTokenRepo for FakeDesktopTokenRepo {
    async fn insert(&self, token: &DesktopToken) -> Result<(), RepoError> {
        let mut tokens = self.tokens.lock().await;
        tokens.insert(token.id, token.clone());
        Ok(())
    }

    async fn find_active_by_hash(&self, hash: &str) -> Result<Option<DesktopToken>, RepoError> {
        let tokens = self.tokens.lock().await;
        Ok(tokens
            .values()
            .find(|t| t.token_hash == hash && t.revoked_at.is_none())
            .cloned())
    }

    async fn touch_last_seen(
        &self,
        id: DesktopTokenId,
        at: DateTime<Utc>,
    ) -> Result<(), RepoError> {
        let mut tokens = self.tokens.lock().await;
        if let Some(t) = tokens.get_mut(&id) {
            t.last_seen_at = Some(at);
        }
        Ok(())
    }

    async fn revoke(&self, id: DesktopTokenId) -> Result<(), RepoError> {
        let mut tokens = self.tokens.lock().await;
        if let Some(t) = tokens.get_mut(&id) {
            t.revoked_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn list_active_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<DesktopToken>, RepoError> {
        let tokens = self.tokens.lock().await;
        Ok(tokens
            .values()
            .filter(|t| t.user_id == user_id && t.revoked_at.is_none())
            .cloned()
            .collect())
    }
}

// ─── FakeDesktopActivityRepo ──────────────────────────────────────────────

pub struct FakeDesktopActivityRepo {
    rows: Arc<Mutex<Vec<DesktopActivityRow>>>,
}

impl Default for FakeDesktopActivityRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeDesktopActivityRepo {
    pub fn new() -> Self {
        Self {
            rows: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl DesktopActivityRepo for FakeDesktopActivityRepo {
    async fn append_batch(&self, batch: &[DesktopActivityRow]) -> Result<(), RepoError> {
        let mut rows = self.rows.lock().await;
        rows.extend_from_slice(batch);
        Ok(())
    }

    async fn prune_before(&self, threshold: DateTime<Utc>) -> Result<u64, RepoError> {
        let mut rows = self.rows.lock().await;
        let before = rows.len();
        rows.retain(|r| r.received_at >= threshold);
        Ok((before - rows.len()) as u64)
    }
}

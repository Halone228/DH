//! Composition root.
//!
//! This is the only place in the codebase that knows about *concrete*
//! adapters. Everywhere else depends on the trait objects defined in
//! `dayhelper-ports`. To swap an adapter (e.g. Postgres for SQLite, or
//! a different notifier), edit this file — nothing in `application/`,
//! `bot/`, `tma/`, `scheduler/`, or `server-desktop-api/` needs to change.

use std::sync::Arc;

use dayhelper_adapter_sqlite::{
    SqliteDesktopActivityRepo, SqliteDesktopTokenRepo, SqliteJobQueue, SqliteNudgeSettingsRepo,
    SqlitePairCodeStore, SqlitePool, SqliteReminderRepo, SqliteUserRepo,
};
use dayhelper_adapter_system::{OsRandom, SystemClock};
use dayhelper_adapter_telegram::TelegramNotifier;
use dayhelper_application::{
    AcceptDesktopSync, CancelReminder, CreateReminder, EnsureUser, FireDueJobs, IssuePairCode,
    ListReminders, PruneOldData, PruneRetention, RedeemPairCode, ScheduleDailyNudges,
    UpdateNudgeSettings, UpdateTimezone,
};
use dayhelper_ports::{
    Clock, DesktopActivityRepo, DesktopTokenRepo, JobQueue, Notifier, NudgeSettingsRepo,
    PairCodeStore, RandomSource, ReminderRepo, UserRepo,
};
use dayhelper_scheduler::Scheduler;
use teloxide::Bot;

use crate::config::Config;

#[allow(dead_code)] // ports/adapters held for future wiring (tests, new transports)
pub struct Container {
    pub config: Arc<Config>,
    pub bot: Bot,

    pub clock: Arc<dyn Clock>,
    pub rng: Arc<dyn RandomSource>,
    pub notifier: Arc<dyn Notifier>,

    pub users: Arc<dyn UserRepo>,
    pub reminders: Arc<dyn ReminderRepo>,
    pub nudge_settings: Arc<dyn NudgeSettingsRepo>,
    pub jobs: Arc<dyn JobQueue>,
    pub desktop_tokens: Arc<dyn DesktopTokenRepo>,
    pub desktop_activity: Arc<dyn DesktopActivityRepo>,
    pub pair_codes: Arc<dyn PairCodeStore>,

    pub ensure_user: Arc<EnsureUser>,
    pub create_reminder: Arc<CreateReminder>,
    pub list_reminders: Arc<ListReminders>,
    pub cancel_reminder: Arc<CancelReminder>,
    pub fire_due_jobs: Arc<FireDueJobs>,
    pub schedule_daily_nudges: Arc<ScheduleDailyNudges>,
    pub issue_pair_code: Arc<IssuePairCode>,
    pub redeem_pair_code: Arc<RedeemPairCode>,
    pub accept_desktop_sync: Arc<AcceptDesktopSync>,
    pub prune_old_data: Arc<PruneOldData>,
    pub update_timezone: Arc<UpdateTimezone>,
    pub update_nudge_settings: Arc<UpdateNudgeSettings>,

    pub scheduler: Arc<Scheduler>,
}

impl Container {
    pub fn build(config: Config, pool: SqlitePool, bot: Bot) -> Self {
        let config = Arc::new(config);

        // Adapter wiring — concrete -> trait object.
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let rng: Arc<dyn RandomSource> = Arc::new(OsRandom);
        let notifier: Arc<dyn Notifier> = Arc::new(TelegramNotifier::new(bot.clone()));

        let users: Arc<dyn UserRepo> = Arc::new(SqliteUserRepo::new(pool.clone()));
        let reminders: Arc<dyn ReminderRepo> = Arc::new(SqliteReminderRepo::new(pool.clone()));
        let nudge_settings: Arc<dyn NudgeSettingsRepo> =
            Arc::new(SqliteNudgeSettingsRepo::new(pool.clone()));
        let jobs: Arc<dyn JobQueue> = Arc::new(SqliteJobQueue::new(pool.clone()));
        let desktop_tokens: Arc<dyn DesktopTokenRepo> =
            Arc::new(SqliteDesktopTokenRepo::new(pool.clone()));
        let desktop_activity: Arc<dyn DesktopActivityRepo> =
            Arc::new(SqliteDesktopActivityRepo::new(pool.clone()));
        let pair_codes: Arc<dyn PairCodeStore> = Arc::new(SqlitePairCodeStore::new(pool));

        // Use cases — pure constructor injection.
        let ensure_user = Arc::new(EnsureUser::new(users.clone()));
        let create_reminder = Arc::new(CreateReminder::new(
            reminders.clone(),
            jobs.clone(),
            clock.clone(),
        ));
        let list_reminders = Arc::new(ListReminders::new(reminders.clone()));
        let cancel_reminder = Arc::new(CancelReminder::new(reminders.clone(), jobs.clone()));
        let fire_due_jobs = Arc::new(FireDueJobs::new(
            jobs.clone(),
            reminders.clone(),
            users.clone(),
            notifier.clone(),
            clock.clone(),
        ));
        let schedule_daily_nudges = Arc::new(ScheduleDailyNudges::new(
            jobs.clone(),
            clock.clone(),
            rng.clone(),
        ));
        let issue_pair_code = Arc::new(IssuePairCode::new(pair_codes.clone(), clock.clone()));
        let redeem_pair_code = Arc::new(RedeemPairCode::new(
            pair_codes.clone(),
            desktop_tokens.clone(),
            users.clone(),
            clock.clone(),
        ));
        let accept_desktop_sync = Arc::new(AcceptDesktopSync::new(
            desktop_activity.clone(),
            jobs.clone(),
            reminders.clone(),
            clock.clone(),
        ));
        let update_timezone = Arc::new(UpdateTimezone::new(users.clone()));
        let update_nudge_settings = Arc::new(UpdateNudgeSettings::new(nudge_settings.clone()));
        let prune_old_data = Arc::new(PruneOldData::new(
            jobs.clone(),
            desktop_activity.clone(),
            clock.clone(),
            PruneRetention::default(),
        ));

        let scheduler = Arc::new(Scheduler::new(
            fire_due_jobs.clone(),
            schedule_daily_nudges.clone(),
            prune_old_data.clone(),
            jobs.clone(),
            users.clone(),
            nudge_settings.clone(),
            clock.clone(),
        ));

        Self {
            config,
            bot,
            clock,
            rng,
            notifier,
            users,
            reminders,
            nudge_settings,
            jobs,
            desktop_tokens,
            desktop_activity,
            pair_codes,
            ensure_user,
            create_reminder,
            list_reminders,
            cancel_reminder,
            fire_due_jobs,
            schedule_daily_nudges,
            issue_pair_code,
            redeem_pair_code,
            accept_desktop_sync,
            prune_old_data,
            update_timezone,
            update_nudge_settings,
            scheduler,
        }
    }
}

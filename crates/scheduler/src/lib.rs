//! Scheduler runtime. One long-running task that:
//!  1. asks `JobQueue::peek_next_fire_at` when the next event is;
//!  2. sleeps until then (or up to a max interval) — interruptable via `wakeup`;
//!  3. drains all due jobs through `FireDueJobs::tick`.
//!
//! A second task replans daily nudges for every active user once an hour.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use dayhelper_application::{FireDueJobs, PruneOldData, ScheduleDailyNudges};
use dayhelper_domain::NudgeSettings;
use dayhelper_ports::{Clock, JobQueue, NudgeSettingsRepo, UserRepo};
use tokio::sync::Notify;
use tokio::time::sleep;
use tracing::{debug, error, info};

/// Hard cap on how long the loop sleeps without re-checking. Belt-and-braces:
/// even if a wakeup signal is missed, the loop self-heals within this window.
const MAX_IDLE_SLEEP: std::time::Duration = std::time::Duration::from_secs(60);

/// Cadence at which we re-plan nudges for every user.
const NUDGE_PLAN_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 60);

/// Cadence at which we prune old fired jobs and desktop activity rows.
const PRUNE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

#[derive(Clone)]
pub struct SchedulerHandle {
    wakeup: Arc<Notify>,
}

impl SchedulerHandle {
    /// Tell the scheduler to recompute its sleep — call this after enqueueing
    /// a job whose `fire_at` is sooner than the current sleep target.
    pub fn wakeup(&self) {
        self.wakeup.notify_one();
    }
}

pub struct Scheduler {
    fire: Arc<FireDueJobs>,
    nudges: Arc<ScheduleDailyNudges>,
    prune: Arc<PruneOldData>,
    queue: Arc<dyn JobQueue>,
    users: Arc<dyn UserRepo>,
    nudge_settings: Arc<dyn NudgeSettingsRepo>,
    clock: Arc<dyn Clock>,
    wakeup: Arc<Notify>,
}

impl Scheduler {
    pub fn new(
        fire: Arc<FireDueJobs>,
        nudges: Arc<ScheduleDailyNudges>,
        prune: Arc<PruneOldData>,
        queue: Arc<dyn JobQueue>,
        users: Arc<dyn UserRepo>,
        nudge_settings: Arc<dyn NudgeSettingsRepo>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            fire,
            nudges,
            prune,
            queue,
            users,
            nudge_settings,
            clock,
            wakeup: Arc::new(Notify::new()),
        }
    }

    pub fn handle(&self) -> SchedulerHandle {
        SchedulerHandle {
            wakeup: self.wakeup.clone(),
        }
    }

    /// Drives all loops. Stops when `shutdown` fires.
    pub async fn run(self: Arc<Self>, mut shutdown: tokio::sync::broadcast::Receiver<()>) {
        let mut fire_shutdown = shutdown.resubscribe();
        let fire = {
            let s = self.clone();
            tokio::spawn(async move { s.fire_loop(&mut fire_shutdown).await })
        };
        let mut plan_shutdown = shutdown.resubscribe();
        let plan = {
            let s = self.clone();
            tokio::spawn(async move { s.nudge_planner_loop(&mut plan_shutdown).await })
        };
        let mut prune_shutdown = shutdown.resubscribe();
        let prune = {
            let s = self.clone();
            tokio::spawn(async move { s.prune_loop(&mut prune_shutdown).await })
        };

        // Wait for shutdown signal, then let the spawned tasks finish.
        let _ = shutdown.recv().await;
        info!("scheduler received shutdown signal");
        let _ = tokio::join!(fire, plan, prune);
    }

    async fn fire_loop(self: Arc<Self>, shutdown: &mut tokio::sync::broadcast::Receiver<()>) {
        loop {
            loop {
                match self.fire.tick().await {
                    Ok(Some(_)) => continue,
                    Ok(None) => break,
                    Err(e) => {
                        error!(error = %e, "fire tick failed");
                        break;
                    }
                }
            }

            let until = match self.queue.peek_next_fire_at().await {
                Ok(Some(at)) => at,
                Ok(None) => self.clock.now() + Duration::from_std(MAX_IDLE_SLEEP).unwrap(),
                Err(e) => {
                    error!(error = %e, "peek failed, backing off");
                    self.clock.now() + Duration::seconds(5)
                }
            };

            let dur = sleep_duration(self.clock.now(), until);
            debug!(?dur, "scheduler sleeping");
            tokio::select! {
                _ = sleep(dur) => {}
                _ = self.wakeup.notified() => {
                    debug!("scheduler woken externally");
                }
                _ = shutdown.recv() => {
                    info!("fire_loop shutting down");
                    return;
                }
            }
        }
    }

    async fn nudge_planner_loop(self: Arc<Self>, shutdown: &mut tokio::sync::broadcast::Receiver<()>) {
        let mut tick = tokio::time::interval(NUDGE_PLAN_INTERVAL);
        loop {
            tokio::select! {
                _ = tick.tick() => {
                    self.plan_nudges_round().await;
                }
                _ = shutdown.recv() => {
                    info!("nudge_planner_loop shutting down");
                    return;
                }
            }
        }
    }

    async fn plan_nudges_round(&self) {
        let users = match self.users.list_with_nudges_enabled().await {
            Ok(u) => u,
            Err(e) => {
                error!(error = %e, "list users failed");
                return;
            }
        };
        info!(count = users.len(), "planning nudges");

        for user in users {
            let settings = match self.nudge_settings.get(user.id).await {
                Ok(Some(s)) => s,
                Ok(None) => NudgeSettings::default_for(user.id),
                Err(e) => {
                    error!(error = %e, user = ?user.id, "load nudge settings failed");
                    continue;
                }
            };

            if !settings.enabled {
                continue;
            }

            if let Err(e) = self
                .nudges
                .execute(user.id, user.timezone, &settings)
                .await
            {
                error!(error = %e, user = ?user.id, "schedule nudges failed");
            }
        }

        self.wakeup.notify_one();
    }

    async fn prune_loop(self: Arc<Self>, shutdown: &mut tokio::sync::broadcast::Receiver<()>) {
        // First tick after a short delay so we don't fight with bootstrap;
        // subsequent ticks every 24h.
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {}
            _ = shutdown.recv() => {
                info!("prune_loop shutting down");
                return;
            }
        }
        let mut tick = tokio::time::interval(PRUNE_INTERVAL);
        loop {
            tokio::select! {
                _ = tick.tick() => {
                    match self.prune.execute().await {
                        Ok(_) => {}
                        Err(e) => error!(error = %e, "prune failed"),
                    }
                }
                _ = shutdown.recv() => {
                    info!("prune_loop shutting down");
                    return;
                }
            }
        }
    }
}

fn sleep_duration(now: DateTime<Utc>, until: DateTime<Utc>) -> std::time::Duration {
    let delta = (until - now).max(Duration::zero());
    let std = delta.to_std().unwrap_or(std::time::Duration::ZERO);
    std.min(MAX_IDLE_SLEEP)
}

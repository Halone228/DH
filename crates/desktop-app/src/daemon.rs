//! Daemon orchestration: spins up tracker, idle, sync, and fire loops as
//! independent tokio tasks coordinated through mpsc channels and a shared
//! [`SessionAggregator`].

use std::sync::Arc;
use std::time::Duration;

use dayhelper_desktop_application::SessionAggregator;
use dayhelper_desktop_domain::{FocusChange, IdleStatus};
use dayhelper_desktop_ports::{IdleDetector, WindowTracker};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::container::DesktopContainer;

pub struct DaemonOptions {
    pub sync_interval: Duration,
}

pub async fn run(container: Arc<DesktopContainer>, opts: DaemonOptions) -> anyhow::Result<()> {
    // Verify auth before launching the rest of the loops; surfaces an
    // actionable error early instead of failing every minute.
    let creds = container.credentials.load().await?;
    if creds.is_none() {
        anyhow::bail!("not paired — run `dayhelper-cli login <code>` first");
    }

    let (focus_tx, focus_rx) = mpsc::channel::<FocusChange>(64);
    let (idle_tx, idle_rx) = mpsc::channel::<IdleStatus>(16);

    let tracker_task = spawn_tracker(container.tracker.clone(), focus_tx);
    let idle_task = spawn_idle(container.idle.clone(), idle_tx);
    let consume_task = spawn_consumer(container.session.clone(), focus_rx, idle_rx);
    let sync_task = spawn_sync(container.clone(), opts.sync_interval);
    let fire_task = spawn_fire(container.clone());

    info!("daemon up");
    tokio::select! {
        r = tracker_task => warn!(?r, "tracker exited"),
        r = idle_task => warn!(?r, "idle exited"),
        _ = consume_task => warn!("consumer exited"),
        _ = sync_task => warn!("sync exited"),
        _ = fire_task => warn!("fire exited"),
        _ = tokio::signal::ctrl_c() => info!("ctrl-c"),
    }
    Ok(())
}

fn spawn_tracker(
    tracker: Arc<dyn WindowTracker>,
    tx: mpsc::Sender<FocusChange>,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        if let Err(e) = tracker.run(tx).await {
            error!(error = %e, "tracker error");
            return Err(anyhow::anyhow!(e.to_string()));
        }
        Ok(())
    })
}

fn spawn_idle(
    idle: Arc<dyn IdleDetector>,
    tx: mpsc::Sender<IdleStatus>,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        if let Err(e) = idle.run(tx).await {
            error!(error = %e, "idle error");
            return Err(anyhow::anyhow!(e.to_string()));
        }
        Ok(())
    })
}

fn spawn_consumer(
    session: Arc<SessionAggregator>,
    mut focus_rx: mpsc::Receiver<FocusChange>,
    mut idle_rx: mpsc::Receiver<IdleStatus>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(change) = focus_rx.recv() => {
                    if let Err(e) = session.on_focus(change).await {
                        warn!(error = %e, "session focus update failed");
                    }
                }
                Some(status) = idle_rx.recv() => {
                    if let Err(e) = session.on_idle(status).await {
                        warn!(error = %e, "session idle update failed");
                    }
                }
                else => break,
            }
        }
    })
}

fn spawn_sync(container: Arc<DesktopContainer>, period: Duration) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut tick = interval(period);
        // First tick fires immediately; we want a small startup grace.
        tick.tick().await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        loop {
            tick.tick().await;
            if let Err(e) = container.sync.execute().await {
                warn!(error = %e, "sync failed");
            }
        }
    })
}

fn spawn_fire(container: Arc<DesktopContainer>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // Polling cadence: 5s is fine — fire_at granularity is minutes, and
        // we're not optimizing for sub-second latency.
        let mut tick = interval(Duration::from_secs(5));
        loop {
            tick.tick().await;
            if let Err(e) = container.fire_due.tick().await {
                warn!(error = %e, "fire tick failed");
            }
        }
    })
}

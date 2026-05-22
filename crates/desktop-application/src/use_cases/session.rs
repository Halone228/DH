use std::sync::Arc;

use chrono::{DateTime, Utc};
use dayhelper_desktop_domain::{ActivityEvent, ActivityEventId, FocusChange, IdleStatus};
use dayhelper_desktop_ports::LocalActivityRepo;
use tracing::debug;

use crate::DesktopError;

/// Stateful aggregator that turns a stream of [`FocusChange`] and
/// [`IdleStatus`] events into closed [`ActivityEvent`] rows.
///
/// Two transitions close a session:
///   - focus changed (different `app_name` or `window_title`);
///   - user went idle.
///
/// Becoming "active again" *after* idle starts a fresh session under whatever
/// app currently holds focus (re-asserted when the window tracker emits its
/// next change — until then we have no `current` and discard the active edge).
pub struct SessionAggregator {
    repo: Arc<dyn LocalActivityRepo>,
    inner: tokio::sync::Mutex<Inner>,
    /// AFK threshold — sessions shorter than this are kept; idle gaps
    /// shorter than this are not split.
    min_session: chrono::Duration,
}

struct Inner {
    current: Option<OpenSession>,
    is_idle: bool,
}

#[derive(Clone)]
struct OpenSession {
    app_name: String,
    window_title: Option<String>,
    started_at: DateTime<Utc>,
}

impl SessionAggregator {
    pub fn new(repo: Arc<dyn LocalActivityRepo>) -> Self {
        Self {
            repo,
            inner: tokio::sync::Mutex::new(Inner {
                current: None,
                is_idle: false,
            }),
            min_session: chrono::Duration::seconds(2),
        }
    }

    pub async fn on_focus(&self, change: FocusChange) -> Result<(), DesktopError> {
        let mut inner = self.inner.lock().await;
        self.close_current(&mut inner, change.at).await?;

        if inner.is_idle {
            // Discard focus changes that happen while idle. We'll pick a fresh
            // session up when the next focus event arrives after active again.
            debug!("focus change while idle, skipping");
            return Ok(());
        }

        inner.current = change.app_name.map(|app| OpenSession {
            app_name: app,
            window_title: change.window_title,
            started_at: change.at,
        });
        Ok(())
    }

    pub async fn on_idle(&self, status: IdleStatus) -> Result<(), DesktopError> {
        let mut inner = self.inner.lock().await;
        match status.idle_since {
            Some(at) => {
                self.close_current(&mut inner, at).await?;
                inner.is_idle = true;
            }
            None => {
                inner.is_idle = false;
                // We don't reopen a session here — wait for the tracker's next
                // focus emission, otherwise we'd guess wrong about which app
                // is now in focus.
            }
        }
        Ok(())
    }

    async fn close_current(
        &self,
        inner: &mut Inner,
        at: DateTime<Utc>,
    ) -> Result<(), DesktopError> {
        let Some(open) = inner.current.take() else {
            return Ok(());
        };
        if at <= open.started_at {
            return Ok(());
        }
        let duration = at - open.started_at;
        if duration < self.min_session {
            // Too short to be interesting; drop it silently.
            return Ok(());
        }

        let event = ActivityEvent {
            id: ActivityEventId::new(),
            app_name: open.app_name,
            window_title: open.window_title,
            started_at: open.started_at,
            ended_at: at,
            synced: false,
        };
        self.repo.append(&event).await?;
        Ok(())
    }
}

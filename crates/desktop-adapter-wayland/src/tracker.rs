//! Active-window tracker via `zwlr_foreign_toplevel_management_v1`.
//!
//! Event accumulation pattern: per-toplevel events (`app_id`, `title`, `state`)
//! are buffered in `pending` and committed on `done`. When the committed
//! state changes which toplevel has the `Activated` flag, we emit one
//! [`FocusChange`].

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use dayhelper_desktop_domain::FocusChange;
use dayhelper_desktop_ports::{TrackerError, WindowTracker};
use tokio::sync::mpsc;
use tracing::{debug, warn};
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::{self as toplevel, ZwlrForeignToplevelHandleV1},
    zwlr_foreign_toplevel_manager_v1::{self as manager, ZwlrForeignToplevelManagerV1},
};

const STATE_ACTIVATED: u32 = 2;

pub struct WaylandWindowTracker;

impl WaylandWindowTracker {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WaylandWindowTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WindowTracker for WaylandWindowTracker {
    async fn run(&self, tx: mpsc::Sender<FocusChange>) -> Result<(), TrackerError> {
        let handle = tokio::task::spawn_blocking(move || -> Result<(), TrackerError> {
            run_blocking(tx)
        });
        handle
            .await
            .map_err(|e| TrackerError::Io(Box::new(e)))?
    }
}

fn run_blocking(tx: mpsc::Sender<FocusChange>) -> Result<(), TrackerError> {
    let conn = Connection::connect_to_env()
        .map_err(|e| TrackerError::Io(Box::new(e)))?;
    let (globals, mut queue) =
        registry_queue_init::<TrackerState>(&conn).map_err(|e| TrackerError::Io(Box::new(e)))?;
    let qh = queue.handle();

    let _manager: ZwlrForeignToplevelManagerV1 = globals
        .bind(&qh, 1..=3, ())
        .map_err(|_| TrackerError::UnsupportedCompositor("zwlr_foreign_toplevel_management_v1"))?;

    let mut state = TrackerState {
        toplevels: HashMap::new(),
        active: None,
        tx,
    };

    loop {
        if state.tx.is_closed() {
            return Ok(());
        }
        queue
            .blocking_dispatch(&mut state)
            .map_err(|e| TrackerError::Io(Box::new(e)))?;
    }
}

struct TrackerState {
    toplevels: HashMap<u32, Toplevel>,
    active: Option<u32>,
    tx: mpsc::Sender<FocusChange>,
}

#[derive(Default)]
struct Toplevel {
    app_id: Option<String>,
    title: Option<String>,
    activated: bool,
    pending_app_id: Option<String>,
    pending_title: Option<String>,
    pending_activated: Option<bool>,
}

impl TrackerState {
    fn commit(&mut self, key: u32) {
        let Some(t) = self.toplevels.get_mut(&key) else {
            return;
        };
        if let Some(v) = t.pending_app_id.take() {
            t.app_id = Some(v);
        }
        if let Some(v) = t.pending_title.take() {
            t.title = Some(v);
        }
        if let Some(v) = t.pending_activated.take() {
            t.activated = v;
        }

        let now_active = if t.activated { Some(key) } else { None };
        match (self.active, now_active) {
            (Some(prev), Some(_)) if prev != key => {
                if let Some(prev_t) = self.toplevels.get_mut(&prev) {
                    prev_t.activated = false;
                }
                self.active = Some(key);
                self.emit_active(key);
            }
            (None, Some(_)) => {
                self.active = Some(key);
                self.emit_active(key);
            }
            (Some(prev), None) if prev == key => {
                self.active = None;
                self.emit_none();
            }
            _ => {}
        }
    }

    fn emit_active(&self, key: u32) {
        let Some(t) = self.toplevels.get(&key) else {
            return;
        };
        let change = FocusChange {
            at: Utc::now(),
            app_name: t.app_id.clone(),
            window_title: t.title.clone(),
        };
        if self.tx.try_send(change).is_err() {
            debug!("focus channel full or closed");
        }
    }

    fn emit_none(&self) {
        let _ = self.tx.try_send(FocusChange {
            at: Utc::now(),
            app_name: None,
            window_title: None,
        });
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for TrackerState {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: <WlRegistry as Proxy>::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for TrackerState {
    fn event(
        _: &mut Self,
        _: &ZwlrForeignToplevelManagerV1,
        event: manager::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            manager::Event::Toplevel { .. } => {
                // The toplevel arrives as a new ZwlrForeignToplevelHandleV1
                // bound by the Wayland machinery; its events route to our
                // Dispatch impl below. Nothing to do here.
            }
            manager::Event::Finished => {}
            _ => {}
        }
    }
}

impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for TrackerState {
    fn event(
        state: &mut Self,
        proxy: &ZwlrForeignToplevelHandleV1,
        event: toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let key = proxy.id().protocol_id();
        let entry = state.toplevels.entry(key).or_default();
        match event {
            toplevel::Event::AppId { app_id } => {
                entry.pending_app_id = Some(app_id);
            }
            toplevel::Event::Title { title } => {
                entry.pending_title = Some(title);
            }
            toplevel::Event::State { state: bytes } => {
                let activated = bytes
                    .chunks_exact(4)
                    .any(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]]) == STATE_ACTIVATED);
                entry.pending_activated = Some(activated);
            }
            toplevel::Event::Done => {
                state.commit(key);
            }
            toplevel::Event::Closed => {
                let was_active = state.active == Some(key);
                state.toplevels.remove(&key);
                if was_active {
                    state.active = None;
                    state.emit_none();
                }
            }
            other => {
                debug!(?other, "unhandled toplevel event");
            }
        }
        // Belt-and-braces: protocols sometimes send events without `done`.
        if state.tx.is_closed() {
            warn!("tracker channel closed");
        }
    }
}

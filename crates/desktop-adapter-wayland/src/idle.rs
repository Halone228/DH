//! User-idle detector via `ext_idle_notify_v1`.

use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use dayhelper_desktop_domain::IdleStatus;
use dayhelper_desktop_ports::{IdleDetector, TrackerError};
use tokio::sync::mpsc;
use tracing::debug;
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1::{self, ExtIdleNotificationV1},
    ext_idle_notifier_v1::ExtIdleNotifierV1,
};

pub struct WaylandIdleDetector {
    timeout: Duration,
}

impl WaylandIdleDetector {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

#[async_trait]
impl IdleDetector for WaylandIdleDetector {
    async fn run(&self, tx: mpsc::Sender<IdleStatus>) -> Result<(), TrackerError> {
        let timeout = self.timeout;
        let handle = tokio::task::spawn_blocking(move || -> Result<(), TrackerError> {
            run_blocking(tx, timeout)
        });
        handle
            .await
            .map_err(|e| TrackerError::Io(Box::new(e)))?
    }
}

fn run_blocking(tx: mpsc::Sender<IdleStatus>, timeout: Duration) -> Result<(), TrackerError> {
    let conn = Connection::connect_to_env()
        .map_err(|e| TrackerError::Io(Box::new(e)))?;
    let (globals, mut queue) =
        registry_queue_init::<IdleState>(&conn).map_err(|e| TrackerError::Io(Box::new(e)))?;
    let qh = queue.handle();

    let notifier: ExtIdleNotifierV1 = globals
        .bind(&qh, 1..=1, ())
        .map_err(|_| TrackerError::UnsupportedCompositor("ext_idle_notifier_v1"))?;

    let seat: WlSeat = globals
        .bind(&qh, 1..=8, ())
        .map_err(|_| TrackerError::UnsupportedCompositor("wl_seat"))?;

    let _notification: ExtIdleNotificationV1 =
        notifier.get_idle_notification(timeout.as_millis() as u32, &seat, &qh, ());

    let mut state = IdleState { tx };

    loop {
        if state.tx.is_closed() {
            return Ok(());
        }
        queue
            .blocking_dispatch(&mut state)
            .map_err(|e| TrackerError::Io(Box::new(e)))?;
    }
}

struct IdleState {
    tx: mpsc::Sender<IdleStatus>,
}

impl Dispatch<WlRegistry, GlobalListContents> for IdleState {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: <WlRegistry as wayland_client::Proxy>::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtIdleNotifierV1, ()> for IdleState {
    fn event(
        _: &mut Self,
        _: &ExtIdleNotifierV1,
        _: <ExtIdleNotifierV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSeat, ()> for IdleState {
    fn event(
        _: &mut Self,
        _: &WlSeat,
        _: <WlSeat as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtIdleNotificationV1, ()> for IdleState {
    fn event(
        state: &mut Self,
        _: &ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_idle_notification_v1::Event::Idled => {
                debug!("user idled");
                let _ = state.tx.try_send(IdleStatus::idle(Utc::now()));
            }
            ext_idle_notification_v1::Event::Resumed => {
                debug!("user resumed");
                let _ = state.tx.try_send(IdleStatus::active());
            }
            _ => {}
        }
    }
}

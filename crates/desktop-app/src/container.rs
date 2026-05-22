//! Composition root for the desktop client. Concrete adapters are named
//! exactly once, here. To swap (e.g. add `desktop-adapter-gnome` for a
//! GNOME tracker), edit only this file.

use std::sync::Arc;
use std::time::Duration;

use dayhelper_desktop_adapter_dbus::DbusNotifier;
use dayhelper_desktop_adapter_http::HttpSyncClient;
use dayhelper_desktop_adapter_sqlite::{
    FileCredentialsStore, SqliteActivityRepo, SqliteNotificationRepo, SqlitePool,
};
use dayhelper_desktop_adapter_wayland::{WaylandIdleDetector, WaylandWindowTracker};
use dayhelper_desktop_application::{
    FireDueLocalNotifications, PairDevice, SessionAggregator, SyncWithServer,
};
use dayhelper_desktop_ports::{
    CredentialsStore, DesktopNotifier, IdleDetector, LocalActivityRepo, LocalNotificationRepo,
    SyncClient, WindowTracker,
};

use crate::paths::DesktopPaths;

#[allow(dead_code)] // ports retained for future wiring (alternative trackers, tests)
pub struct DesktopContainer {
    pub credentials: Arc<dyn CredentialsStore>,
    pub sync_client: Arc<dyn SyncClient>,
    pub notifier: Arc<dyn DesktopNotifier>,
    pub tracker: Arc<dyn WindowTracker>,
    pub idle: Arc<dyn IdleDetector>,
    pub activity: Arc<dyn LocalActivityRepo>,
    pub notifications: Arc<dyn LocalNotificationRepo>,

    pub pair: Arc<PairDevice>,
    pub sync: Arc<SyncWithServer>,
    pub fire_due: Arc<FireDueLocalNotifications>,
    pub session: Arc<SessionAggregator>,
}

impl DesktopContainer {
    pub fn build(
        pool: SqlitePool,
        paths: &DesktopPaths,
        server_url: String,
        idle_after: Duration,
    ) -> anyhow::Result<Self> {
        let credentials: Arc<dyn CredentialsStore> =
            Arc::new(FileCredentialsStore::at(paths.credentials_path()));
        let sync_client: Arc<dyn SyncClient> = Arc::new(
            HttpSyncClient::new(server_url)
                .map_err(|e| anyhow::anyhow!("http client: {e}"))?,
        );
        let notifier: Arc<dyn DesktopNotifier> = Arc::new(DbusNotifier::new("dayhelper"));
        let tracker: Arc<dyn WindowTracker> = Arc::new(WaylandWindowTracker::new());
        let idle: Arc<dyn IdleDetector> = Arc::new(WaylandIdleDetector::new(idle_after));

        let activity: Arc<dyn LocalActivityRepo> =
            Arc::new(SqliteActivityRepo::new(pool.clone()));
        let notifications: Arc<dyn LocalNotificationRepo> =
            Arc::new(SqliteNotificationRepo::new(pool));

        let pair = Arc::new(PairDevice::new(sync_client.clone(), credentials.clone()));
        let sync = Arc::new(SyncWithServer::new(
            sync_client.clone(),
            credentials.clone(),
            activity.clone(),
            notifications.clone(),
        ));
        let fire_due = Arc::new(FireDueLocalNotifications::new(
            notifications.clone(),
            notifier.clone(),
        ));
        let session = Arc::new(SessionAggregator::new(activity.clone()));

        Ok(Self {
            credentials,
            sync_client,
            notifier,
            tracker,
            idle,
            activity,
            notifications,
            pair,
            sync,
            fire_due,
            session,
        })
    }
}

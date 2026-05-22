use std::sync::Arc;

use dayhelper_desktop_domain::{LocalNotification, LocalNotificationState};
use dayhelper_desktop_ports::{
    CredentialsStore, LocalActivityRepo, LocalNotificationRepo, SyncClient,
};
use dayhelper_protocol::{ActivityBatchItem, NotificationDelivery, SyncRequest};
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::DesktopError;

const ACTIVITY_BATCH_LIMIT: u32 = 500;

pub struct SyncWithServer {
    sync: Arc<dyn SyncClient>,
    creds: Arc<dyn CredentialsStore>,
    activity: Arc<dyn LocalActivityRepo>,
    notifications: Arc<dyn LocalNotificationRepo>,
    cursor: Mutex<Option<String>>,
}

impl SyncWithServer {
    pub fn new(
        sync: Arc<dyn SyncClient>,
        creds: Arc<dyn CredentialsStore>,
        activity: Arc<dyn LocalActivityRepo>,
        notifications: Arc<dyn LocalNotificationRepo>,
    ) -> Self {
        Self {
            sync,
            creds,
            activity,
            notifications,
            cursor: Mutex::new(None),
        }
    }

    pub async fn execute(&self) -> Result<(), DesktopError> {
        let creds = self
            .creds
            .load()
            .await?
            .ok_or(DesktopError::NotAuthenticated)?;

        let pending = self.activity.unsynced(ACTIVITY_BATCH_LIMIT).await?;
        let activity_items: Vec<ActivityBatchItem> = pending
            .iter()
            .map(|e| ActivityBatchItem {
                app_name: e.app_name.clone(),
                window_title: e.window_title.clone(),
                started_at: e.started_at,
                ended_at: e.ended_at,
            })
            .collect();
        let activity_ids: Vec<uuid::Uuid> = pending.iter().map(|e| e.id.0).collect();

        let fired_acks = self.notifications.fired_pending_ack().await?;

        let cursor = self.cursor.lock().await.clone();
        let req = SyncRequest {
            since_cursor: cursor,
            activity: activity_items,
            fired_notifications: fired_acks.clone(),
        };

        debug!(activity = req.activity.len(), acks = fired_acks.len(), "sync");
        let resp = self.sync.sync(&creds.token, &req).await?;

        // Server accepted the activity batch + acks.
        self.activity.mark_synced(&activity_ids).await?;
        self.notifications.clear_fired_acks(&fired_acks).await?;
        *self.cursor.lock().await = Some(resp.cursor.clone());

        for n in &resp.notifications {
            self.persist_delivery(n).await?;
        }

        info!(
            cursor = resp.cursor,
            received = resp.notifications.len(),
            "synced"
        );
        Ok(())
    }

    async fn persist_delivery(&self, d: &NotificationDelivery) -> Result<(), DesktopError> {
        // Past-due notifications get filtered into Skipped by the fire loop
        // when they're too old, so we only need to enqueue them here.
        let local = LocalNotification {
            id: d.id,
            title: d.title.clone(),
            body: d.body.clone(),
            fire_at: d.fire_at,
            category: format!("{:?}", d.category).to_lowercase(),
            state: LocalNotificationState::Pending,
            fired_at: None,
        };
        self.notifications.upsert(&local).await?;
        Ok(())
    }
}

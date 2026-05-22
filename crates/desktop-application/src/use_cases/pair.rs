use std::sync::Arc;

use chrono::Utc;
use dayhelper_desktop_domain::{Credentials, DeviceToken};
use dayhelper_desktop_ports::{CredentialsStore, SyncClient};
use dayhelper_protocol::PairRequest;

use crate::DesktopError;

pub struct PairDevice {
    sync: Arc<dyn SyncClient>,
    creds: Arc<dyn CredentialsStore>,
}

impl PairDevice {
    pub fn new(sync: Arc<dyn SyncClient>, creds: Arc<dyn CredentialsStore>) -> Self {
        Self { sync, creds }
    }

    pub async fn execute(
        &self,
        code: String,
        device_label: String,
        server_url: String,
    ) -> Result<Credentials, DesktopError> {
        let resp = self
            .sync
            .pair(&PairRequest { code, device_label })
            .await?;
        let creds = Credentials {
            user_id: resp.user_id,
            server_url,
            token: DeviceToken::new(resp.token),
            paired_at: Utc::now(),
        };
        self.creds.save(&creds).await?;
        Ok(creds)
    }
}

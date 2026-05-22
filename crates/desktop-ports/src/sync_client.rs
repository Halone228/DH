use async_trait::async_trait;
use dayhelper_desktop_domain::DeviceToken;
use dayhelper_protocol::{PairRequest, PairResponse, SyncRequest, SyncResponse};

use crate::errors::SyncError;

#[async_trait]
pub trait SyncClient: Send + Sync {
    async fn pair(&self, req: &PairRequest) -> Result<PairResponse, SyncError>;

    async fn sync(
        &self,
        token: &DeviceToken,
        req: &SyncRequest,
    ) -> Result<SyncResponse, SyncError>;
}

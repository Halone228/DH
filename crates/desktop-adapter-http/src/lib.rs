//! Reqwest-based HTTP client to the dayhelper server.

use async_trait::async_trait;
use dayhelper_desktop_domain::DeviceToken;
use dayhelper_desktop_ports::{SyncClient, SyncError};
use dayhelper_protocol::{
    PairRequest, PairResponse, SyncRequest, SyncResponse, PROTOCOL_VERSION,
};
use reqwest::{Client, StatusCode};
use tracing::debug;

pub struct HttpSyncClient {
    base: String,
    http: Client,
}

impl HttpSyncClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, SyncError> {
        let http = Client::builder()
            .user_agent(concat!("dayhelper-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| SyncError::Transport(Box::new(e)))?;
        Ok(Self {
            base: base_url.into(),
            http,
        })
    }

    fn url(&self, path: &str) -> String {
        let trimmed = self.base.trim_end_matches('/');
        format!("{trimmed}{path}")
    }
}

#[async_trait]
impl SyncClient for HttpSyncClient {
    async fn pair(&self, req: &PairRequest) -> Result<PairResponse, SyncError> {
        let resp = self
            .http
            .post(self.url("/api/desktop/pair"))
            .header("X-Dayhelper-Proto", PROTOCOL_VERSION)
            .json(req)
            .send()
            .await
            .map_err(|e| SyncError::Transport(Box::new(e)))?;
        check_ok(resp).await?.json().await.map_err(transport)
    }

    async fn sync(
        &self,
        token: &DeviceToken,
        req: &SyncRequest,
    ) -> Result<SyncResponse, SyncError> {
        debug!(activity = req.activity.len(), "POST /api/desktop/sync");
        let resp = self
            .http
            .post(self.url("/api/desktop/sync"))
            .header("X-Dayhelper-Proto", PROTOCOL_VERSION)
            .bearer_auth(token.as_str())
            .json(req)
            .send()
            .await
            .map_err(transport)?;
        check_ok(resp).await?.json().await.map_err(transport)
    }
}

async fn check_ok(resp: reqwest::Response) -> Result<reqwest::Response, SyncError> {
    let status = resp.status();
    if status == StatusCode::UNAUTHORIZED {
        return Err(SyncError::Unauthenticated);
    }
    if status.is_success() {
        return Ok(resp);
    }
    let message = resp.text().await.unwrap_or_default();
    Err(SyncError::Rejected {
        status: status.as_u16(),
        message,
    })
}

fn transport(e: reqwest::Error) -> SyncError {
    SyncError::Transport(Box::new(e))
}

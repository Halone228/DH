use async_trait::async_trait;

use crate::errors::NotifyError;

#[async_trait]
pub trait DesktopNotifier: Send + Sync {
    async fn show(&self, title: &str, body: &str) -> Result<(), NotifyError>;
}

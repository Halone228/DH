use async_trait::async_trait;
use dayhelper_domain::TelegramUserId;

use crate::errors::NotifyError;

/// Outbound message channel. Today this is Telegram; tomorrow it could be
/// email/SMS/web-push without touching the application layer.
#[async_trait]
pub trait Notifier: Send + Sync {
    async fn notify(&self, user: TelegramUserId, message: &str) -> Result<(), NotifyError>;
}

use async_trait::async_trait;
use dayhelper_domain::TelegramUserId;
use dayhelper_ports::{Notifier, NotifyError};
use teloxide::prelude::*;
use teloxide::types::ChatId;

pub struct TelegramNotifier {
    bot: Bot,
}

impl TelegramNotifier {
    pub fn new(bot: Bot) -> Self {
        Self { bot }
    }
}

#[async_trait]
impl Notifier for TelegramNotifier {
    async fn notify(&self, user: TelegramUserId, message: &str) -> Result<(), NotifyError> {
        self.bot
            .send_message(ChatId(user.0), message)
            .await
            .map_err(NotifyError::transport)?;
        Ok(())
    }
}

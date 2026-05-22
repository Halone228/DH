use async_trait::async_trait;
use dayhelper_domain::{NudgeSettings, TelegramUserId, User, UserId};

use crate::errors::RepoError;

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn upsert(&self, user: &User) -> Result<(), RepoError>;
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepoError>;
    async fn find_by_telegram_id(
        &self,
        telegram_id: TelegramUserId,
    ) -> Result<Option<User>, RepoError>;
    async fn list_with_nudges_enabled(&self) -> Result<Vec<User>, RepoError>;
}

#[async_trait]
pub trait NudgeSettingsRepo: Send + Sync {
    async fn save(&self, settings: &NudgeSettings) -> Result<(), RepoError>;
    async fn get(&self, user_id: UserId) -> Result<Option<NudgeSettings>, RepoError>;
}

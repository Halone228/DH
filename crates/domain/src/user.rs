use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::ids::{TelegramUserId, UserId};
use crate::locale::Locale;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub telegram_id: TelegramUserId,
    pub username: Option<String>,
    pub timezone: Tz,
    pub locale: Locale,
}

impl User {
    pub fn new(telegram_id: TelegramUserId, timezone: Tz) -> Self {
        Self {
            id: UserId::new(),
            telegram_id,
            username: None,
            timezone,
            locale: Locale::Ru,
        }
    }
}

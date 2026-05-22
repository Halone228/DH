use std::sync::Arc;

use chrono::Duration;
use dayhelper_domain::UserId;
use dayhelper_ports::{Clock, PairCodeStore};

use crate::AppError;

const PAIR_CODE_TTL: Duration = Duration::minutes(5);

pub struct IssuePairCode {
    store: Arc<dyn PairCodeStore>,
    clock: Arc<dyn Clock>,
}

impl IssuePairCode {
    pub fn new(store: Arc<dyn PairCodeStore>, clock: Arc<dyn Clock>) -> Self {
        Self { store, clock }
    }

    /// Returns the human-typeable code. Caller (bot handler) shows it to
    /// the user; user types it into `dayhelper-cli login`.
    pub async fn execute(&self, user_id: UserId) -> Result<String, AppError> {
        let code = self
            .store
            .issue(user_id, PAIR_CODE_TTL, self.clock.now())
            .await?;
        Ok(code)
    }
}

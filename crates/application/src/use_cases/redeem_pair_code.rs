use std::sync::Arc;

use dayhelper_domain::{DesktopToken, DesktopTokenId, User};
use dayhelper_ports::{Clock, DesktopTokenRepo, PairCodeStore, UserRepo};
use rand::RngCore;
use sha2::{Digest, Sha256};
use tracing::info;

use crate::AppError;

pub struct RedeemPairCodeOutcome {
    pub token: String,
    pub user: User,
}

pub struct RedeemPairCode {
    pair_codes: Arc<dyn PairCodeStore>,
    tokens: Arc<dyn DesktopTokenRepo>,
    users: Arc<dyn UserRepo>,
    clock: Arc<dyn Clock>,
}

impl RedeemPairCode {
    pub fn new(
        pair_codes: Arc<dyn PairCodeStore>,
        tokens: Arc<dyn DesktopTokenRepo>,
        users: Arc<dyn UserRepo>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            pair_codes,
            tokens,
            users,
            clock,
        }
    }

    pub async fn execute(
        &self,
        code: &str,
        device_label: String,
    ) -> Result<RedeemPairCodeOutcome, AppError> {
        let now = self.clock.now();
        let user_id = self
            .pair_codes
            .redeem(code, now)
            .await?
            .ok_or_else(|| AppError::Invalid("invalid or expired pair code".into()))?;

        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::NotFound)?;

        let plaintext = mint_token();
        let hash = sha256_hex(&plaintext);

        let record = DesktopToken {
            id: DesktopTokenId::new(),
            user_id,
            token_hash: hash,
            label: device_label,
            created_at: now,
            last_seen_at: None,
            revoked_at: None,
        };
        self.tokens.insert(&record).await?;
        info!(user = ?user_id, "issued desktop token");
        Ok(RedeemPairCodeOutcome {
            token: plaintext,
            user,
        })
    }
}

/// 32 random bytes encoded as URL-safe base64 (no padding) — 43 chars.
fn mint_token() -> String {
    use base64::Engine;
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

pub fn sha256_hex(plaintext: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    hex::encode(hasher.finalize())
}

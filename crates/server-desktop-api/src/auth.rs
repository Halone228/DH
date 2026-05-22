//! Bearer-token extractor.
//!
//! Reads `Authorization: Bearer <token>` header, hashes the token with
//! SHA-256, looks it up in `DesktopTokenRepo`. On hit, exposes the
//! associated `User` to handlers as [`AuthedDesktop`]. Touches `last_seen_at`
//! once per request — best-effort; failures are not fatal.

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use chrono::Utc;
use dayhelper_application::auth::sha256_hex;
use dayhelper_domain::{DesktopToken, User};
use tracing::{debug, warn};

use crate::state::ServerDesktopState;

#[derive(Debug)]
pub struct ApiAuthError {
    pub status: StatusCode,
    pub message: &'static str,
}

impl axum::response::IntoResponse for ApiAuthError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.message).into_response()
    }
}

pub struct AuthedDesktop {
    pub user: User,
    pub token: DesktopToken,
}

#[async_trait::async_trait]
impl FromRequestParts<ServerDesktopState> for AuthedDesktop {
    type Rejection = ApiAuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerDesktopState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiAuthError {
                status: StatusCode::UNAUTHORIZED,
                message: "missing Authorization",
            })?;
        let token = header.strip_prefix("Bearer ").ok_or(ApiAuthError {
            status: StatusCode::UNAUTHORIZED,
            message: "expected `Bearer <token>`",
        })?;

        let hash = sha256_hex(token);
        let token = state
            .tokens
            .find_active_by_hash(&hash)
            .await
            .map_err(|e| {
                warn!(error = %e, "token lookup failed");
                ApiAuthError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "token lookup failed",
                }
            })?
            .ok_or(ApiAuthError {
                status: StatusCode::UNAUTHORIZED,
                message: "unknown token",
            })?;

        // Best-effort touch.
        if let Err(e) = state.tokens.touch_last_seen(token.id, Utc::now()).await {
            debug!(error = %e, "could not update last_seen_at");
        }

        let user = state
            .users
            .find_by_id(token.user_id)
            .await
            .map_err(|e| {
                warn!(error = %e, "user lookup failed");
                ApiAuthError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "user lookup failed",
                }
            })?
            .ok_or(ApiAuthError {
                status: StatusCode::UNAUTHORIZED,
                message: "user gone",
            })?;
        Ok(Self { user, token })
    }
}

//! Telegram Mini App `initData` verification.
//!
//! Spec: https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app
//!
//! Algorithm:
//!   1. Parse the URL-encoded `initData` into key/value pairs.
//!   2. Pull out `hash` — that's what we'll verify.
//!   3. Sort remaining pairs by key, join `key=value` with `\n` → data_check_string.
//!   4. secret_key = HMAC_SHA256("WebAppData", bot_token)
//!   5. expected   = HMAC_SHA256(secret_key, data_check_string)
//!   6. Compare in constant time with the supplied `hash`.

use std::collections::BTreeMap;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use chrono::{TimeZone, Utc};
use dayhelper_domain::{TelegramUserId, User};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use tracing::warn;

use crate::state::TmaState;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct InitDataError {
    pub status: StatusCode,
    pub message: &'static str,
}

impl axum::response::IntoResponse for InitDataError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.message).into_response()
    }
}

/// Parse + verify `initData`. Returns the sender's Telegram user id.
pub fn verify(init_data: &str, bot_token: &str) -> Result<TelegramUserId, InitDataError> {
    let mut pairs = parse_init_data(init_data).map_err(|m| InitDataError {
        status: StatusCode::BAD_REQUEST,
        message: m,
    })?;

    let supplied_hash = pairs.remove("hash").ok_or(InitDataError {
        status: StatusCode::UNAUTHORIZED,
        message: "missing hash",
    })?;

    let data_check_string = pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut secret = HmacSha256::new_from_slice(b"WebAppData").expect("hmac key");
    secret.update(bot_token.as_bytes());
    let secret_key = secret.finalize().into_bytes();

    let mut expected = HmacSha256::new_from_slice(&secret_key).expect("hmac key");
    expected.update(data_check_string.as_bytes());
    let expected_hex = hex::encode(expected.finalize().into_bytes());

    if !constant_time_eq(expected_hex.as_bytes(), supplied_hash.as_bytes()) {
        return Err(InitDataError {
            status: StatusCode::UNAUTHORIZED,
            message: "bad signature",
        });
    }

    // Reject stale initData (> 24 h old).
    if let Some(auth_date_str) = pairs.get("auth_date") {
        let auth_date_ts: i64 = auth_date_str
            .parse()
            .map_err(|_| InitDataError {
                status: StatusCode::BAD_REQUEST,
                message: "invalid auth_date",
            })?;
        let auth_date = Utc
            .timestamp_opt(auth_date_ts, 0)
            .single()
            .ok_or(InitDataError {
                status: StatusCode::BAD_REQUEST,
                message: "invalid auth_date timestamp",
            })?;
        let age = Utc::now() - auth_date;
        if age.num_hours() > 24 {
            return Err(InitDataError {
                status: StatusCode::UNAUTHORIZED,
                message: "initData expired",
            });
        }
    }

    let user_blob = pairs.get("user").ok_or(InitDataError {
        status: StatusCode::BAD_REQUEST,
        message: "missing user",
    })?;
    let info: TmaUser = serde_json::from_str(user_blob).map_err(|_| InitDataError {
        status: StatusCode::BAD_REQUEST,
        message: "user not json",
    })?;
    Ok(TelegramUserId(info.id))
}

fn parse_init_data(s: &str) -> Result<BTreeMap<String, String>, &'static str> {
    let parsed = url::form_urlencoded::parse(s.as_bytes());
    let mut map = BTreeMap::new();
    for (k, v) in parsed {
        map.insert(k.into_owned(), v.into_owned());
    }
    if map.is_empty() {
        return Err("empty initData");
    }
    Ok(map)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[derive(Deserialize)]
struct TmaUser {
    id: i64,
}

/// Axum extractor that verifies `initData` from the `Authorization: tma <data>`
/// header (or the `X-Init-Data` header) and resolves the corresponding `User`.
pub struct AuthedUser(pub User);

#[async_trait::async_trait]
impl FromRequestParts<TmaState> for AuthedUser {
    type Rejection = InitDataError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &TmaState,
    ) -> Result<Self, Self::Rejection> {
        let init_data = extract_init_data(parts).ok_or(InitDataError {
            status: StatusCode::UNAUTHORIZED,
            message: "missing initData",
        })?;

        let tg_id = verify(&init_data, &state.bot_token)?;
        let user = state
            .ensure_user
            .execute(tg_id, state.default_timezone)
            .await
            .map_err(|e| {
                warn!(error = %e, "ensure_user failed");
                InitDataError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "ensure_user failed",
                }
            })?;
        Ok(AuthedUser(user))
    }
}

fn extract_init_data(parts: &Parts) -> Option<String> {
    if let Some(v) = parts.headers.get("authorization") {
        let s = v.to_str().ok()?;
        if let Some(rest) = s.strip_prefix("tma ") {
            return Some(rest.to_string());
        }
    }
    parts
        .headers
        .get("x-init-data")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

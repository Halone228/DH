use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use dayhelper_application::AppError;
use dayhelper_protocol::{PairRequest, PairResponse, SyncRequest, SyncResponse};
use tower_http::trace::TraceLayer;
use tracing::warn;

use crate::auth::AuthedDesktop;
use crate::state::ServerDesktopState;

/// Global rate limiter for `/api/desktop/pair`: 10 attempts per minute.
/// Pair codes are 6-digit — limiting brute-force is critical.
static PAIR_LIMITER: OnceLock<Mutex<HashMap<i64, (Instant, u32)>>> = OnceLock::new();
const PAIR_MAX: u32 = 10;
const PAIR_WINDOW: Duration = Duration::from_secs(60);
/// Global key — pair endpoint is unauthenticated, so we rate-limit by a
/// fixed sentinel to cap total attempts across all callers.
const PAIR_GLOBAL_KEY: i64 = 0;

pub fn build_router(state: ServerDesktopState) -> Router {
    Router::new()
        .route("/api/desktop/pair", post(pair))
        .route("/api/desktop/sync", post(sync))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

async fn pair(
    State(state): State<ServerDesktopState>,
    Json(req): Json<PairRequest>,
) -> Result<Json<PairResponse>, ApiError> {
    // Rate-limit pair attempts (brute-force protection for 6-digit codes).
    let limiter = PAIR_LIMITER.get_or_init(|| Mutex::new(HashMap::new()));
    {
        let mut map = limiter.lock().unwrap();
        let now = Instant::now();
        let entry = map.entry(PAIR_GLOBAL_KEY).or_insert((now, 0));
        if now.duration_since(entry.0) > PAIR_WINDOW {
            *entry = (now, 1);
        } else if entry.1 < PAIR_MAX {
            entry.1 += 1;
        } else {
            return Err(ApiError(AppError::Invalid(
                "Too many pairing attempts. Try again later.".into(),
            )));
        }
    }

    let outcome = state
        .redeem_pair_code
        .execute(&req.code, req.device_label)
        .await?;
    Ok(Json(PairResponse {
        token: outcome.token,
        user_id: outcome.user.id.0,
        expires_at: None,
    }))
}

async fn sync(
    State(state): State<ServerDesktopState>,
    AuthedDesktop { user, token: _ }: AuthedDesktop,
    Json(req): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, ApiError> {
    let resp = state.accept_sync.execute(&user, req).await?;
    Ok(Json(resp))
}

struct ApiError(AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        ApiError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self.0 {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Invalid(_) => StatusCode::BAD_REQUEST,
            AppError::Storage(_) | AppError::Notify(_) => {
                warn!(error = %self.0, "internal");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (status, self.0.to_string()).into_response()
    }
}

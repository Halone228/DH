use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use chrono::NaiveTime;
use dayhelper_application::{AppError, CreateReminderCommand};
use dayhelper_domain::{Recurrence, ReminderId, Weekday};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::auth::AuthedUser;
use crate::state::TmaState;

pub fn build_router(state: TmaState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/reminders", get(list_reminders).post(create_reminder))
        .route("/api/reminders/:id", delete(cancel_reminder))
        .route("/api/me", get(me).patch(update_me))
        .route("/api/nudge-settings", get(get_nudge_settings).put(update_nudge_settings))
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(Any).allow_headers(Any).allow_methods(Any))
        .layer(TraceLayer::new_for_http())
}

async fn health() -> &'static str {
    "ok"
}

// ─── Me ─────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct MeResponse {
    user_id: Uuid,
    telegram_id: i64,
    timezone: String,
    is_new: bool,
}

async fn me(AuthedUser { user, is_new }: AuthedUser) -> Json<MeResponse> {
    Json(MeResponse {
        user_id: user.id.0,
        telegram_id: user.telegram_id.0,
        timezone: user.timezone.name().to_string(),
        is_new,
    })
}

#[derive(Deserialize)]
struct UpdateMeRequest {
    timezone: Option<String>,
}

async fn update_me(
    State(state): State<TmaState>,
    AuthedUser { user, is_new: _ }: AuthedUser,
    Json(body): Json<UpdateMeRequest>,
) -> Result<Json<MeResponse>, ApiError> {
    if let Some(ref tz) = body.timezone {
        state.update_timezone.execute(user.id, tz).await?;
    }
    // Re-fetch is unnecessary — timezone came back from the user object we
    // already have if it didn't change. When it *did* change, parse the
    // validated string back.
    let updated_tz = body
        .timezone
        .map(|s| s.parse().expect("validated by use case"))
        .unwrap_or(user.timezone);
    Ok(Json(MeResponse {
        user_id: user.id.0,
        telegram_id: user.telegram_id.0,
        timezone: updated_tz.name().to_string(),
        is_new: false,
    }))
}

// ─── Reminders ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ReminderDto {
    id: Uuid,
    text: String,
    recurrence: Recurrence,
    active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn list_reminders(
    State(state): State<TmaState>,
    AuthedUser { user, is_new: _ }: AuthedUser,
) -> Result<Json<Vec<ReminderDto>>, ApiError> {
    let items = state.list_reminders.execute(user.id).await?;
    let dto = items
        .into_iter()
        .map(|r| ReminderDto {
            id: r.id.0,
            text: r.text,
            recurrence: r.recurrence,
            active: r.active,
            created_at: r.created_at,
        })
        .collect();
    Ok(Json(dto))
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RecurrenceDto {
    Once { at: chrono::DateTime<chrono::Utc> },
    Daily { time: NaiveTime },
    Weekly { weekdays: Vec<Weekday>, time: NaiveTime },
    Monthly { day_of_month: u8, time: NaiveTime },
}

impl From<RecurrenceDto> for Recurrence {
    fn from(d: RecurrenceDto) -> Self {
        match d {
            RecurrenceDto::Once { at } => Recurrence::Once { at },
            RecurrenceDto::Daily { time } => Recurrence::Daily { time },
            RecurrenceDto::Weekly { weekdays, time } => Recurrence::Weekly { weekdays, time },
            RecurrenceDto::Monthly { day_of_month, time } => {
                Recurrence::Monthly { day_of_month, time }
            }
        }
    }
}

#[derive(Deserialize)]
struct CreateReminderRequest {
    text: String,
    recurrence: RecurrenceDto,
}

async fn create_reminder(
    State(state): State<TmaState>,
    AuthedUser { user, is_new: _ }: AuthedUser,
    Json(req): Json<CreateReminderRequest>,
) -> Result<Json<ReminderDto>, ApiError> {
    let r = state
        .create_reminder
        .execute(CreateReminderCommand {
            user_id: user.id,
            user_timezone: user.timezone,
            text: req.text,
            recurrence: req.recurrence.into(),
        })
        .await?;
    state.scheduler.wakeup();
    Ok(Json(ReminderDto {
        id: r.id.0,
        text: r.text,
        recurrence: r.recurrence,
        active: r.active,
        created_at: r.created_at,
    }))
}

async fn cancel_reminder(
    State(state): State<TmaState>,
    AuthedUser { user, is_new: _ }: AuthedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // Ownership check: verify the reminder belongs to the authenticated user.
    let reminder = state
        .reminder_repo
        .get(ReminderId(id))
        .await
        .map_err(|e| ApiError(AppError::Storage(e)))?
        .ok_or_else(|| ApiError(AppError::NotFound))?;
    if reminder.user_id != user.id {
        return Err(ApiError(AppError::NotFound));
    }
    state.cancel_reminder.execute(ReminderId(id)).await?;
    state.scheduler.wakeup();
    Ok(StatusCode::NO_CONTENT)
}

// ─── Nudge settings ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct NudgeSettingsDto {
    enabled: bool,
    daily_count: u8,
    active_window_start: String,
    active_window_end: String,
}

impl From<dayhelper_domain::NudgeSettings> for NudgeSettingsDto {
    fn from(s: dayhelper_domain::NudgeSettings) -> Self {
        Self {
            enabled: s.enabled,
            daily_count: s.daily_count,
            active_window_start: s.active_window_start.format("%H:%M:%S").to_string(),
            active_window_end: s.active_window_end.format("%H:%M:%S").to_string(),
        }
    }
}

async fn get_nudge_settings(
    State(state): State<TmaState>,
    AuthedUser { user, is_new: _ }: AuthedUser,
) -> Result<Json<NudgeSettingsDto>, ApiError> {
    let settings = state.update_nudge_settings.get(user.id).await?;
    Ok(Json(NudgeSettingsDto::from(settings)))
}

#[derive(Deserialize)]
struct UpdateNudgeSettingsRequest {
    enabled: Option<bool>,
    daily_count: Option<u8>,
    active_window_start: Option<String>,
    active_window_end: Option<String>,
}

async fn update_nudge_settings(
    State(state): State<TmaState>,
    AuthedUser { user, is_new: _ }: AuthedUser,
    Json(body): Json<UpdateNudgeSettingsRequest>,
) -> Result<Json<NudgeSettingsDto>, ApiError> {
    if let Some(count) = body.daily_count {
        if !(1..=20).contains(&count) {
            return Err(ApiError(AppError::Invalid(
                "daily_count must be between 1 and 20".into(),
            )));
        }
        state
            .update_nudge_settings
            .set_daily_count(user.id, count)
            .await?;
    }

    if let Some(enabled) = body.enabled {
        state
            .update_nudge_settings
            .set_enabled(user.id, enabled)
            .await?;
    }

    // Window is updated atomically (both start and end together).
    match (body.active_window_start, body.active_window_end) {
        (Some(start_s), Some(end_s)) => {
            let start = parse_naive_time(&start_s)?;
            let end = parse_naive_time(&end_s)?;
            state
                .update_nudge_settings
                .set_window(user.id, start, end)
                .await?;
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(ApiError(AppError::Invalid(
                "both active_window_start and active_window_end must be provided together".into(),
            )));
        }
        (None, None) => {}
    }

    let settings = state.update_nudge_settings.get(user.id).await?;
    Ok(Json(NudgeSettingsDto::from(settings)))
}

fn parse_naive_time(s: &str) -> Result<NaiveTime, AppError> {
    NaiveTime::parse_from_str(s, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M"))
        .map_err(|_| AppError::Invalid("invalid time format, expected HH:MM:SS or HH:MM".into()))
}

// ─── Error type ─────────────────────────────────────────────────────────────

struct ApiError(AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        ApiError(e)
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self.0 {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Invalid(_) => StatusCode::BAD_REQUEST,
            AppError::Storage(_) | AppError::Notify(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.0.to_string()).into_response()
    }
}

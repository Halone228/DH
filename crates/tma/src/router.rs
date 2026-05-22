use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use chrono::{DateTime, NaiveTime, Utc};
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
        .route("/api/me", get(me))
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(Any).allow_headers(Any).allow_methods(Any))
        .layer(TraceLayer::new_for_http())
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Serialize)]
struct MeResponse {
    user_id: Uuid,
    telegram_id: i64,
    timezone: String,
}

async fn me(AuthedUser(user): AuthedUser) -> Json<MeResponse> {
    Json(MeResponse {
        user_id: user.id.0,
        telegram_id: user.telegram_id.0,
        timezone: user.timezone.name().to_string(),
    })
}

#[derive(Serialize)]
struct ReminderDto {
    id: Uuid,
    text: String,
    recurrence: Recurrence,
    active: bool,
    created_at: DateTime<Utc>,
}

async fn list_reminders(
    State(state): State<TmaState>,
    AuthedUser(user): AuthedUser,
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
    Once { at: DateTime<Utc> },
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
    AuthedUser(user): AuthedUser,
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
    AuthedUser(_user): AuthedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.cancel_reminder.execute(ReminderId(id)).await?;
    state.scheduler.wakeup();
    Ok(StatusCode::NO_CONTENT)
}

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

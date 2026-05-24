use std::sync::Arc;

use chrono_tz::Tz;
use dayhelper_application::{
    CancelReminder, CreateReminder, EnsureUser, ListReminders, UpdateNudgeSettings,
    UpdateTimezone,
};
use dayhelper_ports::ReminderRepo;
use dayhelper_scheduler::SchedulerHandle;

#[derive(Clone)]
pub struct TmaState {
    pub bot_token: Arc<str>,
    pub default_timezone: Tz,
    pub ensure_user: Arc<EnsureUser>,
    pub create_reminder: Arc<CreateReminder>,
    pub list_reminders: Arc<ListReminders>,
    pub cancel_reminder: Arc<CancelReminder>,
    pub update_timezone: Arc<UpdateTimezone>,
    pub update_nudge_settings: Arc<UpdateNudgeSettings>,
    pub reminder_repo: Arc<dyn ReminderRepo>,
    pub scheduler: SchedulerHandle,
}

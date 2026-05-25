//! Teloxide command handlers. The bot crate intentionally owns no state of
//! its own — every dependency is passed in via [`BotDeps`] from the
//! composition root, mirroring the DI pattern used everywhere else.

use std::sync::Arc;

use chrono_tz::Tz;
use chrono::{Offset, TimeZone};
use dayhelper_application::{
    l10n::BotMessages,
    CancelReminder, CreateReminder, CreateReminderCommand, EnsureResult, EnsureUser,
    IssuePairCode, ListReminders, UpdateNudgeSettings, UpdateTimezone,
};
use dayhelper_domain::{Recurrence, ReminderId, Weekday};
use dayhelper_scheduler::SchedulerHandle;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::utils::command::BotCommands;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct BotDeps {
    pub ensure_user: Arc<EnsureUser>,
    pub create_reminder: Arc<CreateReminder>,
    pub list_reminders: Arc<ListReminders>,
    pub cancel_reminder: Arc<CancelReminder>,
    pub issue_pair_code: Arc<IssuePairCode>,
    pub update_timezone: Arc<UpdateTimezone>,
    pub update_nudge_settings: Arc<UpdateNudgeSettings>,
    pub scheduler: SchedulerHandle,
    pub default_timezone: Tz,
    pub tma_url: String,
}

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Bot commands:")]
pub enum Command {
    #[command(description = "greeting and app link")]
    Start,
    #[command(description = "list reminders")]
    List,
    #[command(description = "one-shot reminder: /once 2026-05-04T10:00 text")]
    Once(String),
    #[command(description = "daily reminder: /daily 09:00 text")]
    Daily(String),
    #[command(description = "weekly reminder: /weekly Mon,Wed,Fri 09:00 text")]
    Weekly(String),
    #[command(description = "monthly reminder: /monthly 15 09:00 text")]
    Monthly(String),
    #[command(description = "cancel reminder: /cancel <id>")]
    Cancel(String),
    #[command(description = "get desktop pairing code")]
    Pair,
    #[command(description = "change timezone: /timezone Europe/Moscow")]
    Timezone(String),
    #[command(description = "toggle nudges: /nudge on or /nudge off")]
    Nudge(String),
    #[command(description = "nudge window: /nudge_window 09:00 21:00")]
    NudgeWindow(String),
    #[command(description = "show current settings")]
    Settings,
    #[command(description = "help")]
    Help,
}

/// Register bot commands with Telegram so users see autocomplete.
pub async fn setup_commands(bot: &Bot) -> anyhow::Result<()> {
    bot.set_my_commands(Command::bot_commands()).await?;
    tracing::info!("bot commands registered");
    Ok(())
}

/// Build the dispatcher. The caller spawns `dispatch().await`.
pub fn build_dispatcher(bot: Bot, deps: BotDeps) -> Dispatcher<Bot, anyhow::Error, teloxide::dispatching::DefaultKey> {
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(handle_command),
        )
        .branch(
            Update::filter_callback_query()
                .endpoint(handle_callback),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![deps])
        .build()
}

#[tracing::instrument(skip(bot, deps, msg))]
async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    deps: BotDeps,
) -> anyhow::Result<()> {
    let Some(from) = msg.from.as_ref() else {
        return Ok(());
    };
    let telegram_id = dayhelper_domain::TelegramUserId(from.id.0 as i64);
    let user = match deps
        .ensure_user
        .execute(telegram_id, deps.default_timezone)
        .await
    {
        Ok(u) => u,
        Err(e) => return reply_error(&bot, msg.chat.id, e).await,
    };
    let is_new = matches!(user, EnsureResult::New(_));
    let user = user.user().clone();
    let l = BotMessages::for_locale(user.locale);
    tracing::Span::current().record("user_id", user.id.0.to_string());
    tracing::Span::current().record("cmd", format!("{cmd:?}"));

    match cmd {
        Command::Start => {
            let text = if is_new {
                l.start_new.replace("{}", &deps.tma_url)
            } else {
                l.start_existing.replace("{}", &deps.tma_url)
            };
            bot.send_message(msg.chat.id, text).await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::List => {
            let items = match deps
                .list_reminders
                .execute(user.id)
                .await
            {
                Ok(items) => items,
                Err(e) => return reply_error(&bot, msg.chat.id, e).await,
            };
            if items.is_empty() {
                bot.send_message(msg.chat.id, l.list_empty).await?;
            } else {
                let keyboard: Vec<Vec<InlineKeyboardButton>> = items
                    .iter()
                    .map(|r| {
                        vec![InlineKeyboardButton::callback(
                            format!("❌ {} — {}", short_id(r.id.0), r.text),
                            format!("cancel:{}", r.id.0),
                        )]
                    })
                    .collect();
                bot.send_message(msg.chat.id, l.list_header)
                    .reply_markup(InlineKeyboardMarkup::new(keyboard))
                    .await?;
            }
        }
        Command::Once(args) => {
            match parse_once(&args, user.timezone) {
                Ok((at_utc, text)) => {
                    let r = match deps
                        .create_reminder
                        .execute(CreateReminderCommand {
                            user_id: user.id,
                            user_timezone: user.timezone,
                            text,
                            recurrence: Recurrence::Once { at: at_utc },
                        })
                        .await
                    {
                        Ok(r) => r,
                        Err(e) => return reply_error(&bot, msg.chat.id, e).await,
                    };
                    deps.scheduler.wakeup();
                    bot.send_message(
                        msg.chat.id,
                        l.reminder_created.replace("{}", &short_id(r.id.0)),
                    )
                    .await?;
                }
                Err(e) => {
                    bot.send_message(msg.chat.id, l.parse_once_hint.replace("{}", &e.to_string()))
                        .await?;
                }
            }
        }
        Command::Daily(args) => match parse_daily(&args) {
            Ok((time, text)) => {
                let r = match deps
                    .create_reminder
                    .execute(CreateReminderCommand {
                        user_id: user.id,
                        user_timezone: user.timezone,
                        text,
                        recurrence: Recurrence::Daily { time },
                    })
                    .await
                {
                    Ok(r) => r,
                    Err(e) => return reply_error(&bot, msg.chat.id, e).await,
                };
                deps.scheduler.wakeup();
                bot.send_message(
                    msg.chat.id,
                    l.daily_created.replace("{}", &short_id(r.id.0)),
                )
                .await?;
            }
            Err(e) => {
                bot.send_message(msg.chat.id, l.parse_daily_hint.replace("{}", &e.to_string()))
                    .await?;
            }
        },
        Command::Weekly(args) => match parse_weekly(&args) {
            Ok((weekdays, time, text)) => {
                let r = match deps
                    .create_reminder
                    .execute(CreateReminderCommand {
                        user_id: user.id,
                        user_timezone: user.timezone,
                        text,
                        recurrence: Recurrence::Weekly { weekdays, time },
                    })
                    .await
                {
                    Ok(r) => r,
                    Err(e) => return reply_error(&bot, msg.chat.id, e).await,
                };
                deps.scheduler.wakeup();
                bot.send_message(
                    msg.chat.id,
                    l.weekly_created.replace("{}", &short_id(r.id.0)),
                )
                .await?;
            }
            Err(e) => {
                bot.send_message(
                    msg.chat.id,
                    l.parse_weekly_hint.replace("{}", &e.to_string()),
                )
                .await?;
            }
        },
        Command::Monthly(args) => match parse_monthly(&args) {
            Ok((day, time, text)) => {
                let r = match deps
                    .create_reminder
                    .execute(CreateReminderCommand {
                        user_id: user.id,
                        user_timezone: user.timezone,
                        text,
                        recurrence: Recurrence::Monthly {
                            day_of_month: day,
                            time,
                        },
                    })
                    .await
                {
                    Ok(r) => r,
                    Err(e) => return reply_error(&bot, msg.chat.id, e).await,
                };
                deps.scheduler.wakeup();
                bot.send_message(
                    msg.chat.id,
                    l.monthly_created.replace("{}", &short_id(r.id.0)),
                )
                .await?;
            }
            Err(e) => {
                bot.send_message(
                    msg.chat.id,
                    l.parse_monthly_hint.replace("{}", &e.to_string()),
                )
                .await?;
            }
        },
        Command::Pair => {
            let code = match deps.issue_pair_code.execute(user.id).await {
                Ok(code) => code,
                Err(e) => return reply_error(&bot, msg.chat.id, e).await,
            };
            bot.send_message(
                msg.chat.id,
                l.pair_success
                    .replacen("{}", &code, 1)
                    .replacen("{}", &code, 1),
            )
            .await?;
        }
        Command::Cancel(arg) => {
            let trimmed = arg.trim();
            match Uuid::parse_str(trimmed) {
                Ok(id) => {
                    if let Err(e) = deps.cancel_reminder.execute(ReminderId(id)).await {
                        return reply_error(&bot, msg.chat.id, e).await;
                    }
                    deps.scheduler.wakeup();
                    bot.send_message(msg.chat.id, l.reminder_cancelled).await?;
                }
                Err(_) => {
                    bot.send_message(msg.chat.id, l.need_full_uuid)
                        .await?;
                }
            }
        }
        Command::Timezone(arg) => {
            let tz_str = arg.trim();
            if tz_str.is_empty() {
                bot.send_message(msg.chat.id, l.timezone_specify)
                    .await?;
                return Ok(());
            }
            match deps.update_timezone.execute(user.id, tz_str).await {
                Ok(()) => {
                    bot.send_message(
                        msg.chat.id,
                        l.timezone_updated.replace("{}", tz_str),
                    )
                    .await?;
                }
                Err(dayhelper_application::AppError::Invalid(_)) => {
                    bot.send_message(
                        msg.chat.id,
                        l.timezone_invalid.replace("{}", tz_str),
                    )
                    .await?;
                }
                Err(e) => return reply_error(&bot, msg.chat.id, e).await,
            }
        }
        Command::Nudge(arg) => {
            let val = arg.trim().to_lowercase();
            let enabled = match val.as_str() {
                "on" | "вкл" | "1" => true,
                "off" | "выкл" | "0" => false,
                _ => {
                    bot.send_message(msg.chat.id, l.nudge_specify)
                        .await?;
                    return Ok(());
                }
            };
            if let Err(e) = deps.update_nudge_settings
                .set_enabled(user.id, enabled)
                .await
            {
                return reply_error(&bot, msg.chat.id, e).await;
            }
            let label = if enabled {
                l.nudge_enabled
            } else {
                l.nudge_disabled
            };
            bot.send_message(msg.chat.id, label).await?;
        }
        Command::NudgeWindow(arg) => {
            let mut parts = arg.trim().splitn(2, char::is_whitespace);
            let start_str = parts.next().unwrap_or("");
            let end_str = parts.next().unwrap_or("");
            let start = match chrono::NaiveTime::parse_from_str(start_str, "%H:%M") {
                Ok(t) => t,
                Err(_) => {
                    bot.send_message(msg.chat.id, l.nudge_window_invalid)
                        .await?;
                    return Ok(());
                }
            };
            let end = match chrono::NaiveTime::parse_from_str(end_str, "%H:%M") {
                Ok(t) => t,
                Err(_) => {
                    bot.send_message(msg.chat.id, l.nudge_window_invalid)
                        .await?;
                    return Ok(());
                }
            };
            match deps.update_nudge_settings.set_window(user.id, start, end).await {
                Ok(()) => {
                    bot.send_message(
                        msg.chat.id,
                        l.nudge_window_updated
                            .replacen("{}", &start.format("%H:%M").to_string(), 1)
                            .replacen("{}", &end.format("%H:%M").to_string(), 1),
                    )
                    .await?;
                }
                Err(dayhelper_application::AppError::Invalid(err_msg)) => {
                    bot.send_message(msg.chat.id, err_msg).await?;
                }
                Err(e) => return reply_error(&bot, msg.chat.id, e).await,
            }
        }
        Command::Settings => {
            let settings = match deps
                .update_nudge_settings
                .get(user.id)
                .await
            {
                Ok(s) => s,
                Err(e) => return reply_error(&bot, msg.chat.id, e).await,
            };
            let tz_offset = chrono::Utc::now().with_timezone(&user.timezone).offset().fix();
            let offset_str = format!(
                "{:+}{}",
                tz_offset.local_minus_utc() / 3600,
                if tz_offset.local_minus_utc() % 3600 != 0 {
                    format!(":{:02}", (tz_offset.local_minus_utc() % 3600).abs() / 60)
                } else {
                    String::new()
                }
            );
            let enabled_label = if settings.enabled {
                l.settings_nudge_on
            } else {
                l.settings_nudge_off
            };
            let text = format!(
                "{}\n\n{}: {} (UTC{})\n{}: {}\n{}: {}\n{}: {} — {}",
                l.settings_header,
                l.settings_tz,
                user.timezone.name(),
                offset_str,
                l.settings_nudge,
                enabled_label,
                l.settings_count,
                settings.daily_count,
                l.settings_window,
                settings.active_window_start.format("%H:%M"),
                settings.active_window_end.format("%H:%M"),
            );
            bot.send_message(msg.chat.id, text).await?;
        }
    }

    Ok(())
}

async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    deps: BotDeps,
) -> anyhow::Result<()> {
    let Some(data) = q.data else { return Ok(()) };
    let Some(message) = q.message else { return Ok(()) };

    if let Some(uuid_str) = data.strip_prefix("cancel:") {
        match Uuid::parse_str(uuid_str) {
            Ok(id) => {
                // Resolve the user from the callback sender.
                let tg_id = dayhelper_domain::TelegramUserId(q.from.id.0 as i64);
                let user = match deps
                    .ensure_user
                    .execute(tg_id, deps.default_timezone)
                    .await
                {
                    Ok(u) => u,
                    Err(e) => return reply_error(&bot, message.chat().id, e).await,
                };
                let user = user.user();
                let l = BotMessages::for_locale(user.locale);

                // Ownership check: only cancel reminders belonging to this user.
                let items = match deps.list_reminders.execute(user.id).await {
                    Ok(items) => items,
                    Err(e) => return reply_error(&bot, message.chat().id, e).await,
                };
                if !items.iter().any(|r| r.id.0 == id) {
                    bot.answer_callback_query(&q.id).text(l.not_found).await?;
                    return Ok(());
                }

                if let Err(e) = deps.cancel_reminder.execute(ReminderId(id)).await {
                    return reply_error(&bot, message.chat().id, e).await;
                }
                deps.scheduler.wakeup();

                bot.answer_callback_query(&q.id)
                    .text(l.callback_cancelled)
                    .await?;
                bot.edit_message_text(
                    message.chat().id,
                    message.id(),
                    l.callback_cancelled_msg,
                )
                .await?;
            }
            Err(_) => {
                bot.answer_callback_query(&q.id).text("Error").await?;
            }
        }
    }
    Ok(())
}

pub(crate) fn parse_once(args: &str, tz: Tz) -> Result<(chrono::DateTime<chrono::Utc>, String), String> {
    let mut parts = args.splitn(2, char::is_whitespace);
    let datetime = parts.next().ok_or("no date")?.trim();
    let text = parts.next().ok_or("no text")?.trim().to_string();
    if text.is_empty() {
        return Err("empty text".into());
    }
    let naive = chrono::NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M")
        .map_err(|e| e.to_string())?;
    // Interpret naive datetime in the user's local timezone, mirroring the
    // combine() DST pattern from domain::recurrence.
    let aware = match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(dt, _) => dt, // earliest, same as combine()
        chrono::LocalResult::None => {
            // DST gap: bump forward by 1 minute and retry
            let bumped = naive + chrono::Duration::minutes(1);
            tz.from_local_datetime(&bumped)
                .single()
                .ok_or("could not resolve time (DST transition)".to_string())?
        }
    };
    Ok((aware.with_timezone(&chrono::Utc), text))
}

pub(crate) fn parse_daily(args: &str) -> Result<(chrono::NaiveTime, String), String> {
    let mut parts = args.splitn(2, char::is_whitespace);
    let time = parts.next().ok_or("no time")?.trim();
    let text = parts.next().ok_or("no text")?.trim().to_string();
    if text.is_empty() {
        return Err("empty text".into());
    }
    let t = chrono::NaiveTime::parse_from_str(time, "%H:%M").map_err(|e| e.to_string())?;
    Ok((t, text))
}

pub(crate) fn parse_weekly(args: &str) -> Result<(Vec<Weekday>, chrono::NaiveTime, String), String> {
    let mut parts = args.splitn(3, char::is_whitespace);
    let weekdays_str = parts.next().ok_or("no weekdays")?.trim();
    let time_str = parts.next().ok_or("no time")?.trim();
    let text = parts.next().ok_or("no text")?.trim().to_string();
    if text.is_empty() {
        return Err("empty text".into());
    }
    let weekdays: Vec<Weekday> = weekdays_str
        .split(',')
        .map(|s| parse_weekday(s.trim()).ok_or_else(|| format!("unknown day: {s}")))
        .collect::<Result<_, _>>()?;
    if weekdays.is_empty() {
        return Err("specify at least one weekday".into());
    }
    let time = chrono::NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|e| format!("invalid time: {e}"))?;
    Ok((weekdays, time, text))
}

pub(crate) fn parse_monthly(args: &str) -> Result<(u8, chrono::NaiveTime, String), String> {
    let mut parts = args.splitn(3, char::is_whitespace);
    let day_str = parts.next().ok_or("no day of month")?.trim();
    let time_str = parts.next().ok_or("no time")?.trim();
    let text = parts.next().ok_or("no text")?.trim().to_string();
    if text.is_empty() {
        return Err("empty text".into());
    }
    let day: u8 = day_str
        .parse()
        .map_err(|_| "day of month must be a number 1-31".to_string())?;
    if !(1..=31).contains(&day) {
        return Err("day of month must be 1-31".into());
    }
    let time = chrono::NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|e| format!("invalid time: {e}"))?;
    Ok((day, time, text))
}

pub(crate) fn parse_weekday(s: &str) -> Option<Weekday> {
    match s.to_lowercase().as_str() {
        "mon" | "пн" => Some(Weekday::Mon),
        "tue" | "вт" => Some(Weekday::Tue),
        "wed" | "ср" => Some(Weekday::Wed),
        "thu" | "чт" => Some(Weekday::Thu),
        "fri" | "пт" => Some(Weekday::Fri),
        "sat" | "сб" => Some(Weekday::Sat),
        "sun" | "вс" => Some(Weekday::Sun),
        _ => None,
    }
}

fn short_id(uuid: Uuid) -> String {
    uuid.to_string()[..8].to_string()
}

/// Map an application error to a user-friendly message.
/// For locale-aware errors, callers should use `BotMessages` directly.
fn format_app_error_default(e: &dayhelper_application::AppError) -> String {
    match e {
        dayhelper_application::AppError::NotFound => "Not found.".to_string(),
        dayhelper_application::AppError::Invalid(msg) => msg.clone(),
        dayhelper_application::AppError::Storage(_) | dayhelper_application::AppError::Notify(_) => {
            "⚠️ An error occurred. Please try again later.".to_string()
        }
    }
}

/// Log error and reply to the user with a friendly message.
/// Returns `Ok(())` so teloxide does not retry.
async fn reply_error(bot: &Bot, chat_id: ChatId, e: dayhelper_application::AppError) -> anyhow::Result<()> {
    error!(error = %e, "app error");
    let _ = bot.send_message(chat_id, format_app_error_default(&e)).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Europe::Moscow;
    use chrono::{Timelike, Datelike};

    #[test]
    fn test_parse_once_valid() {
        let (dt, text) = parse_once("2026-06-15T10:00 hello world", Moscow).unwrap();
        let local = dt.with_timezone(&Moscow);
        assert_eq!(local.hour(), 10);
        assert_eq!(local.day(), 15);
        assert_eq!(text, "hello world");
    }

    #[test]
    fn test_parse_daily_valid() {
        let (time, text) = parse_daily("09:00 зарядка").unwrap();
        assert_eq!(time.hour(), 9);
        assert_eq!(text, "зарядка");
    }

    #[test]
    fn test_parse_weekly_en() {
        let (weekdays, time, text) = parse_weekly("Mon,Wed,Fri 09:00 gym").unwrap();
        assert_eq!(weekdays, vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]);
        assert_eq!(time.hour(), 9);
        assert_eq!(text, "gym");
    }

    #[test]
    fn test_parse_weekly_ru() {
        let (weekdays, _, _) = parse_weekly("пн,ср,пт 09:00 зарядка").unwrap();
        assert_eq!(weekdays, vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]);
    }

    #[test]
    fn test_parse_monthly_valid() {
        let (day, time, text) = parse_monthly("15 09:30 оплатить счёт").unwrap();
        assert_eq!(day, 15);
        assert_eq!(time.hour(), 9);
        assert_eq!(text, "оплатить счёт");
    }

    #[test]
    fn test_parse_monthly_invalid_day() {
        let result = parse_monthly("32 09:00 test");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_weekday_en() {
        assert_eq!(parse_weekday("Mon"), Some(Weekday::Mon));
        assert_eq!(parse_weekday("Tue"), Some(Weekday::Tue));
        assert_eq!(parse_weekday("Sun"), Some(Weekday::Sun));
    }

    #[test]
    fn test_parse_weekday_ru() {
        assert_eq!(parse_weekday("пн"), Some(Weekday::Mon));
        assert_eq!(parse_weekday("ср"), Some(Weekday::Wed));
        assert_eq!(parse_weekday("вс"), Some(Weekday::Sun));
    }

    #[test]
    fn test_parse_once_missing_text() {
        let result = parse_once("2026-06-15T10:00", Moscow);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_daily_invalid_time() {
        let result = parse_daily("25:00 test");
        assert!(result.is_err());
    }
}

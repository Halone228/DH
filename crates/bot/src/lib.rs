//! Teloxide command handlers. The bot crate intentionally owns no state of
//! its own — every dependency is passed in via [`BotDeps`] from the
//! composition root, mirroring the DI pattern used everywhere else.

use std::sync::Arc;

use chrono_tz::Tz;
use chrono::TimeZone;
use dayhelper_application::{
    CancelReminder, CreateReminder, CreateReminderCommand, EnsureUser, IssuePairCode,
    ListReminders,
};
use dayhelper_domain::{Recurrence, ReminderId};
use dayhelper_scheduler::SchedulerHandle;
use teloxide::prelude::*;
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
    pub scheduler: SchedulerHandle,
    pub default_timezone: Tz,
    pub tma_url: String,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды бота:")]
pub enum Command {
    #[command(description = "приветствие и открытие приложения")]
    Start,
    #[command(description = "показать список напоминаний")]
    List,
    #[command(description = "разовое напоминание: /once 2026-05-04T10:00 текст")]
    Once(String),
    #[command(description = "ежедневное напоминание: /daily 09:00 текст")]
    Daily(String),
    #[command(description = "отменить напоминание: /cancel <id>")]
    Cancel(String),
    #[command(description = "получить код для подключения desktop-клиента")]
    Pair,
    #[command(description = "помощь")]
    Help,
}

/// Build the dispatcher. The caller spawns `dispatch().await`.
pub fn build_dispatcher(bot: Bot, deps: BotDeps) -> Dispatcher<Bot, anyhow::Error, teloxide::dispatching::DefaultKey> {
    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(handle_command),
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![deps])
        .build()
}

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
    let user = deps
        .ensure_user
        .execute(telegram_id, deps.default_timezone)
        .await
        .map_err(into_anyhow)?;

    match cmd {
        Command::Start => {
            let text = format!(
                "Привет! Я помогу с напоминаниями и не дам прокрастинировать.\n\nОткрой приложение: {}",
                deps.tma_url
            );
            bot.send_message(msg.chat.id, text).await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::List => {
            let items = deps
                .list_reminders
                .execute(user.id)
                .await
                .map_err(into_anyhow)?;
            let text = if items.is_empty() {
                "Активных напоминаний нет.".to_string()
            } else {
                let lines: Vec<String> = items
                    .iter()
                    .map(|r| format!("• {} — {}", short_id(r.id.0), r.text))
                    .collect();
                lines.join("\n")
            };
            bot.send_message(msg.chat.id, text).await?;
        }
        Command::Once(args) => {
            match parse_once(&args, user.timezone) {
                Ok((at_utc, text)) => {
                    let r = deps
                        .create_reminder
                        .execute(CreateReminderCommand {
                            user_id: user.id,
                            user_timezone: user.timezone,
                            text,
                            recurrence: Recurrence::Once { at: at_utc },
                        })
                        .await
                        .map_err(into_anyhow)?;
                    deps.scheduler.wakeup();
                    bot.send_message(
                        msg.chat.id,
                        format!("Создано напоминание {}", short_id(r.id.0)),
                    )
                    .await?;
                }
                Err(e) => {
                    bot.send_message(msg.chat.id, format!("Не понял: {e}\nПример: /once 2026-05-04T10:00 позвонить маме"))
                        .await?;
                }
            }
        }
        Command::Daily(args) => match parse_daily(&args) {
            Ok((time, text)) => {
                let r = deps
                    .create_reminder
                    .execute(CreateReminderCommand {
                        user_id: user.id,
                        user_timezone: user.timezone,
                        text,
                        recurrence: Recurrence::Daily { time },
                    })
                    .await
                    .map_err(into_anyhow)?;
                deps.scheduler.wakeup();
                bot.send_message(
                    msg.chat.id,
                    format!("Ежедневное напоминание {}", short_id(r.id.0)),
                )
                .await?;
            }
            Err(e) => {
                bot.send_message(msg.chat.id, format!("Не понял: {e}\nПример: /daily 09:00 зарядка"))
                    .await?;
            }
        },
        Command::Pair => {
            let code = deps
                .issue_pair_code
                .execute(user.id)
                .await
                .map_err(into_anyhow)?;
            bot.send_message(
                msg.chat.id,
                format!(
                    "Код для подключения desktop-клиента (действует 5 минут):\n\n  {code}\n\nВведи на устройстве:\n  dayhelper-cli login {code}"
                ),
            )
            .await?;
        }
        Command::Cancel(arg) => {
            let trimmed = arg.trim();
            match Uuid::parse_str(trimmed) {
                Ok(id) => {
                    deps.cancel_reminder
                        .execute(ReminderId(id))
                        .await
                        .map_err(into_anyhow)?;
                    deps.scheduler.wakeup();
                    bot.send_message(msg.chat.id, "Отменено.").await?;
                }
                Err(_) => {
                    bot.send_message(msg.chat.id, "Нужен полный UUID напоминания.")
                        .await?;
                }
            }
        }
    }

    Ok(())
}

fn parse_once(args: &str, tz: Tz) -> Result<(chrono::DateTime<chrono::Utc>, String), String> {
    let mut parts = args.splitn(2, char::is_whitespace);
    let datetime = parts.next().ok_or("нет даты")?.trim();
    let text = parts.next().ok_or("нет текста")?.trim().to_string();
    if text.is_empty() {
        return Err("пустой текст".into());
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
                .ok_or("не удалось разрешить время (переход DST)".to_string())?
        }
    };
    Ok((aware.with_timezone(&chrono::Utc), text))
}

fn parse_daily(args: &str) -> Result<(chrono::NaiveTime, String), String> {
    let mut parts = args.splitn(2, char::is_whitespace);
    let time = parts.next().ok_or("нет времени")?.trim();
    let text = parts.next().ok_or("нет текста")?.trim().to_string();
    if text.is_empty() {
        return Err("пустой текст".into());
    }
    let t = chrono::NaiveTime::parse_from_str(time, "%H:%M").map_err(|e| e.to_string())?;
    Ok((t, text))
}

fn short_id(uuid: Uuid) -> String {
    uuid.to_string()[..8].to_string()
}

fn into_anyhow(e: dayhelper_application::AppError) -> anyhow::Error {
    error!(error = %e, "app error");
    anyhow::Error::msg(e.to_string())
}

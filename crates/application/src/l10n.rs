//! Bot message catalog — all user-facing strings keyed by locale.
//!
//! To add a language: add a match arm in `for_locale()` + fill in all keys
//! in a new static `Messages`.

use dayhelper_domain::Locale;

/// Bot messages — locale-aware. Every field is a template; `{}` positional
/// args are used by `format!()` at the call site.
pub struct BotMessages {
    // Start
    pub start_new: &'static str,
    pub start_existing: &'static str,
    // Commands — reminders
    pub reminder_created: &'static str,
    pub weekly_created: &'static str,
    pub monthly_created: &'static str,
    pub daily_created: &'static str,
    // Commands — cancel / list
    pub reminder_cancelled: &'static str,
    pub need_full_uuid: &'static str,
    pub list_empty: &'static str,
    pub list_header: &'static str,
    pub callback_cancelled: &'static str,
    pub callback_cancelled_msg: &'static str,
    pub not_found: &'static str,
    pub callback_error: &'static str,
    // Commands — pair
    pub pair_success: &'static str,
    // Commands — timezone
    pub timezone_updated: &'static str,
    pub timezone_specify: &'static str,
    pub timezone_invalid: &'static str,
    // Commands — nudge toggle
    pub nudge_specify: &'static str,
    pub nudge_enabled: &'static str,
    pub nudge_disabled: &'static str,
    // Commands — nudge window
    pub nudge_window_invalid: &'static str,
    pub nudge_window_updated: &'static str,
    // Commands — settings
    pub settings_header: &'static str,
    pub settings_tz: &'static str,
    pub settings_nudge: &'static str,
    pub settings_nudge_on: &'static str,
    pub settings_nudge_off: &'static str,
    pub settings_count: &'static str,
    pub settings_window: &'static str,
    // Errors
    pub error_internal: &'static str,
    pub error_not_found: &'static str,
    // Parse errors
    pub parse_once_hint: &'static str,
    pub parse_daily_hint: &'static str,
    pub parse_weekly_hint: &'static str,
    pub parse_monthly_hint: &'static str,
}

impl BotMessages {
    pub fn for_locale(locale: Locale) -> &'static Self {
        match locale {
            Locale::Ru => &RU,
            Locale::En => &EN,
        }
    }
}

static RU: BotMessages = BotMessages {
    start_new:
        "Привет! 👋 Я DayHelper — напоминания и анти-прокрастинация.\n\n\
         📱 Мини-приложение: {}\n\
         🖥️ Desktop-клиент: отправь /pair\n\n\
         Все команды: /help",
    start_existing: "С возвращением! Открой приложение: {}",

    reminder_created: "Создано напоминание {}",
    daily_created: "Ежедневное напоминание {}",
    weekly_created: "Еженедельное напоминание {}",
    monthly_created: "Ежемесячное напоминание {}",

    reminder_cancelled: "Отменено.",
    need_full_uuid: "Нужен полный UUID напоминания.",
    list_empty: "Активных напоминаний нет.",
    list_header: "Напоминания (нажми чтобы отменить):",
    callback_cancelled: "Отменено ✅",
    callback_cancelled_msg: "Напоминание отменено.",
    not_found: "Не найдено",
    callback_error: "Ошибка",

    pair_success:
        "Код для подключения desktop-клиента (действует 5 минут):\n\n\
         \x20 {}\n\n\
         Введи на устройстве:\n\
         \x20 dayhelper-cli login {}",

    timezone_updated: "Часовой пояс обновлён: {}",
    timezone_specify: "Укажите часовой пояс.\nПример: /timezone Europe/Moscow",
    timezone_invalid: "Неверный часовой пояс: {}\nПример: /timezone Europe/Moscow",

    nudge_specify: "Укажите on или off.\nПример: /nudge on",
    nudge_enabled: "Анти-прокрастинация включены.",
    nudge_disabled: "Анти-прокрастинация выключены.",

    nudge_window_invalid: "Неверный формат времени.\nПример: /nudge_window 09:00 21:00",
    nudge_window_updated: "Окно обновлено: {} — {}",

    settings_header: "││͟ Настройки",
    settings_tz: "Часовой пояс",
    settings_nudge: "Анти-прокрастинация",
    settings_nudge_on: "включена",
    settings_nudge_off: "выключена",
    settings_count: "Количество в день",
    settings_window: "Активное окно",

    error_internal: "⚠️ Произошла ошибка. Попробуйте позже.",
    error_not_found: "Не найдено.",

    parse_once_hint: "Не понял: {}\nПример: /once 2026-05-04T10:00 позвонить маме",
    parse_daily_hint: "Не понял: {}\nПример: /daily 09:00 зарядка",
    parse_weekly_hint: "Не понял: {}\nПример: /weekly Mon,Wed,Fri 09:00 зарядка",
    parse_monthly_hint: "Не понял: {}\nПример: /monthly 15 09:00 оплатить счёт",
};

static EN: BotMessages = BotMessages {
    start_new:
        "Hi! 👋 I'm DayHelper — reminders and anti-procrastination.\n\n\
         📱 Mini App: {}\n\
         🖥️ Desktop client: send /pair\n\n\
         All commands: /help",
    start_existing: "Welcome back! Open the app: {}",

    reminder_created: "Reminder created {}",
    daily_created: "Daily reminder {}",
    weekly_created: "Weekly reminder {}",
    monthly_created: "Monthly reminder {}",

    reminder_cancelled: "Cancelled.",
    need_full_uuid: "Full reminder UUID required.",
    list_empty: "No active reminders.",
    list_header: "Reminders (tap to cancel):",
    callback_cancelled: "Cancelled ✅",
    callback_cancelled_msg: "Reminder cancelled.",
    not_found: "Not found",
    callback_error: "Error",

    pair_success:
        "Desktop client pairing code (valid 5 minutes):\n\n\
         \x20 {}\n\n\
         Run on your device:\n\
         \x20 dayhelper-cli login {}",

    timezone_updated: "Timezone updated: {}",
    timezone_specify: "Specify a timezone.\nExample: /timezone Europe/Moscow",
    timezone_invalid: "Invalid timezone: {}\nExample: /timezone Europe/Moscow",

    nudge_specify: "Specify on or off.\nExample: /nudge on",
    nudge_enabled: "Anti-procrastination enabled.",
    nudge_disabled: "Anti-procrastination disabled.",

    nudge_window_invalid: "Invalid time format.\nExample: /nudge_window 09:00 21:00",
    nudge_window_updated: "Window updated: {} — {}",

    settings_header: "││͟ Settings",
    settings_tz: "Timezone",
    settings_nudge: "Anti-procrastination",
    settings_nudge_on: "on",
    settings_nudge_off: "off",
    settings_count: "Daily count",
    settings_window: "Active window",

    error_internal: "⚠️ An error occurred. Please try again later.",
    error_not_found: "Not found.",

    parse_once_hint: "Didn't understand: {}\nExample: /once 2026-05-04T10:00 call mom",
    parse_daily_hint: "Didn't understand: {}\nExample: /daily 09:00 workout",
    parse_weekly_hint: "Didn't understand: {}\nExample: /weekly Mon,Wed,Fri 09:00 workout",
    parse_monthly_hint: "Didn't understand: {}\nExample: /monthly 15 09:00 pay bills",
};

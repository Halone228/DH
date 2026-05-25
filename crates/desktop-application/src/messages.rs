//! Desktop CLI message catalog.
//!
//! Centralises every user-facing string so that adding a locale later is
//! just a matter of adding another `static` and a match arm in
//! [`Messages::for_locale`].

use std::fmt::Display;

use dayhelper_desktop_domain::Locale;

// ── Catalog struct ──────────────────────────────────────────────────────

pub struct Messages {
    // login
    pub login_success: &'static str,
    pub login_next_step: &'static str,
    // daemon
    pub daemon_not_paired: &'static str,
    pub daemon_step1: &'static str,
    pub daemon_step2: &'static str,
    pub daemon_step3: &'static str,
    pub daemon_step4: &'static str,
    // status
    pub status_paired: &'static str,
    pub status_not_paired: &'static str,
    // logout
    pub logout_success: &'static str,
}

impl Messages {
    pub fn for_locale(locale: Locale) -> &'static Self {
        match locale {
            Locale::En => &EN,
            Locale::Ru => &RU,
        }
    }

    /// Returns the login-success line with `{}` replaced by `user_id`.
    pub fn format_login_success(&self, user_id: impl Display) -> String {
        self.login_success.replace("{}", &user_id.to_string())
    }

    /// Returns the status-paired line with `{}` replaced by `user_id`.
    pub fn format_status_paired(&self, user_id: impl Display) -> String {
        self.status_paired.replace("{}", &user_id.to_string())
    }
}

// ── English ─────────────────────────────────────────────────────────────

static EN: Messages = Messages {
    login_success: "✓ Paired successfully (user {}).",
    login_next_step: "Next steps:\n  dayhelper-cli daemon\n\nFor autostart:\n  cp contrib/dayhelper-daemon.service ~/.config/systemd/user/\n  systemctl --user enable --now dayhelper-daemon",
    daemon_not_paired: "Not paired.",
    daemon_step1: "  1. Open your DayHelper bot in Telegram",
    daemon_step2: "  2. Send /pair",
    daemon_step3: "  3. Run: dayhelper-cli login <code>",
    daemon_step4: "  4. Run: dayhelper-cli daemon",
    status_paired: "Paired as user {}",
    status_not_paired: "Not paired. Run 'dayhelper-cli login <code>' first.",
    logout_success: "Credentials removed.",
};

// ── Russian ─────────────────────────────────────────────────────────────

static RU: Messages = Messages {
    login_success: "✓ Успешно подключено (пользователь {}).",
    login_next_step: "Дальнейшие шаги:\n  dayhelper-cli daemon\n\nДля автозапуска:\n  cp contrib/dayhelper-daemon.service ~/.config/systemd/user/\n  systemctl --user enable --now dayhelper-daemon",
    daemon_not_paired: "Устройство не привязано.",
    daemon_step1: "  1. Откройте бота DayHelper в Telegram",
    daemon_step2: "  2. Отправьте /pair",
    daemon_step3: "  3. Выполните: dayhelper-cli login <code>",
    daemon_step4: "  4. Выполните: dayhelper-cli daemon",
    status_paired: "Привязано как пользователь {}",
    status_not_paired: "Не привязано. Сначала выполните 'dayhelper-cli login <code>'.",
    logout_success: "Учётные данные удалены.",
};

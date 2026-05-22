//! Anti-procrastination message bank.
//!
//! Kept here (not in the bot crate) because the *content* is part of the
//! product, not the transport. A future text-localization layer can plug in
//! by replacing this module behind a trait.

use rand_lite::pick;

mod rand_lite {
    /// Tiny helper that picks an index from `0..len` without pulling another
    /// rand dependency into the application crate. Caller supplies the entropy.
    pub fn pick<T>(slice: &[T], seed: u64) -> Option<&T> {
        if slice.is_empty() {
            return None;
        }
        let idx = (seed as usize) % slice.len();
        slice.get(idx)
    }
}

const MESSAGES_RU: &[&str] = &[
    "Хватит прокрастинировать. Открой задачу и сделай один шаг прямо сейчас.",
    "Маленькое действие > идеальный план. Что ты можешь сделать за 5 минут?",
    "Заметил, что ты залип? Закрой вкладку, которая мешает. Возвращайся к делу.",
    "Pomodoro: 25 минут фокуса. Стартуй сейчас.",
    "Один маленький коммит лучше нуля. Поехали.",
    "Что важнее всего сделать сегодня? Сделай это сейчас.",
    "Ты обещал себе. Начни прямо с этой минуты.",
];

/// Returns a nudge text. `seed` should be derived from per-fire entropy
/// (e.g. job id) so two consecutive fires don't pick the same line.
pub fn nudge_text(seed: u64) -> &'static str {
    pick(MESSAGES_RU, seed).copied().unwrap_or(MESSAGES_RU[0])
}

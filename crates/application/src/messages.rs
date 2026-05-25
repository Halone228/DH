//! Anti-procrastination message bank.
//!
//! Kept here (not in the bot crate) because the *content* is part of the
//! product, not the transport. A future text-localization layer can plug in
//! by replacing this module behind a trait.

use dayhelper_domain::Locale;

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
    // ── Action-oriented ──────────────────────────────────────────────────
    "Выбери одну задачу и потрать на неё 5 минут. Только одну.",
    "Открой список задач. Просто открой — это уже начало.",
    "Сделай самое маленькое действие из списка. Даже крошечное.",
    "Закрой всё лишнее. Открой одну задачу. Погнали.",
    "Поставь таймер на 10 минут и работай. Только 10 минут.",
    "Напиши одно предложение по задаче. Одно — уже прогресс.",
    "Открой задачу, которую откладываешь дольше всего. Начни.",
    "Разбей задачу на три шага. Сделай первый.",
    "Сделай сейчас то, что займёт меньше двух минут.",
    "Скопируй задачу в чат с собой. Теперь она перед глазами.",
    "Выбери задачу. Засеки 15 минут. Не отвлекайся.",
    "Сделай один звонок или одно сообщение по задаче. Сейчас.",
    "Открой файл, проект, документ. Просто открой.",
    "Сколько задач можно закрыть за 20 минут? Давай проверим.",
    "Найди задачу, которую можно делегировать. Передай её.",

    // ── Reframing ────────────────────────────────────────────────────────
    "Прокрастинация — это не лень. Это страх. Сделай маленький шаг.",
    "Ты не откладываешь — ты готовишься. Но пора переходить к действию.",
    "Идеальный момент — это сейчас. Не завтра, не после кофе.",
    "Перфекционизм — враг сделанного. Сделай криво, потом поправишь.",
    "Страх ошибки держит тебя на месте. Ошибиться — нормально.",
    "Каждый раз, когда ты откладываешь, задача становится тяжелее.",
    "Ты уже делал сложные вещи раньше. Получится и сейчас.",
    "Не жди мотивации. Она придёт, когда начнёшь.",
    "Задача кажется огромной? Это нормально. Начни с одного куска.",
    "Ты не должен сделать всё. Просто что-то.",

    // ── Time-aware ───────────────────────────────────────────────────────
    "Утро — лучшее время для сложных задач. Используй его.",
    "День в разгаре — самое время сделать один рывок.",
    "Ещё есть время сегодня закрыть хотя бы одну задачу.",
    "Вечер подходит. Сделай одно дело, чтобы завтра было легче.",
    "Сколько часов осталось сегодня? Хватит на одну задачу.",

    // ── Accountability ───────────────────────────────────────────────────
    "Ты обещал себе это сделать. Время пришло.",
    "Завтрашний ты скажет спасибо сегодняшнему. Сделай это.",
    "Кто-то прямо сейчас делает то, что ты откладываешь. Твоя очередь.",
    "Каждый отложенный день — это упущенная возможность.",
    "Представь, как будет приятно вычеркнуть задачу из списка.",
    "Ты заслуживаешь чувство выполненного долга. Начни.",
    "Чем дольше ждёшь, тем сложнее начать. Действуй.",
    "Твоя будущая благодарность начинается с одного шага сейчас.",
    "Сделай это для себя. Не для оценки, не для дедлайна.",
    "Задача не уйдёт. Лучше разобраться с ней сейчас.",

    // ── Gentle humor ─────────────────────────────────────────────────────
    "Открой задачу. Нет, не ту с котиками.",
    "Ты знаешь, что надо делать. Мы оба знаем.",
    "Телефон можно использовать не только для скроллинга. Попробуй.",
    "Ты уже прочитал это сообщение. Теперь открой задачу.",
    "Прокрастинация — тоже навык. Может, пора освоить продуктивность?",
    "Да, это нудж. Нет, я не уйду, пока ты не начнёшь.",
    "Ты можешь закрыть это уведомление. Или закрыть задачу. Выбирай.",
    "Этот нудж исчезнет, но задача — нет.",
    "Скроллишь? А задача ждёт. Она терпеливая, но не бесконечная.",
    "Ну давай, ещё одну задачку. Ты же не из тех, кто сдаётся.",
];

const MESSAGES_EN: &[&str] = &[
    // ── Action-oriented ──────────────────────────────────────────────────
    "Pick one task and spend 5 minutes on it. Just one.",
    "Open your task list. Just open it — that's already a start.",
    "Do the smallest thing on your list. Even a tiny one.",
    "Close everything extra. Open one task. Let's go.",
    "Set a timer for 10 minutes and work. Only 10 minutes.",
    "Write one sentence about the task. One is already progress.",
    "Open the task you've been putting off the longest. Start.",
    "Break the task into three steps. Do the first one.",
    "Do it now if it takes less than two minutes.",
    "Copy the task into a chat with yourself. Now it's in front of you.",
    "Pick a task. Set 15 minutes. Don't get distracted.",
    "Make one call or message about a task. Right now.",
    "Open the file, project, document. Just open it.",
    "How many tasks can you close in 20 minutes? Let's find out.",
    "Find a task you can delegate. Hand it off.",

    // ── Reframing ────────────────────────────────────────────────────────
    "Procrastination isn't laziness. It's fear. Take a small step.",
    "You're not putting it off — you're preparing. But it's time to act.",
    "The perfect moment is now. Not tomorrow, not after coffee.",
    "Perfectionism is the enemy of done. Do it messy, fix it later.",
    "Fear of failure is holding you in place. Mistakes are normal.",
    "Every time you put it off, the task gets heavier.",
    "You've done hard things before. You can do it again.",
    "Don't wait for motivation. It comes after you start.",
    "Task seems huge? That's normal. Start with one piece.",
    "You don't have to do everything. Just something.",

    // ── Time-aware ───────────────────────────────────────────────────────
    "Morning is the best time for hard tasks. Use it.",
    "The day is in full swing — perfect time for one push.",
    "There's still time to close at least one task today.",
    "Evening is here. Do one thing to make tomorrow easier.",
    "How many hours are left today? Enough for one task.",

    // ── Accountability ───────────────────────────────────────────────────
    "You promised yourself you'd do this. Time's up.",
    "Tomorrow-you will thank today-you. Do it.",
    "Someone else is doing what you're putting off right now. Your turn.",
    "Every delayed day is a missed opportunity.",
    "Imagine how good it'll feel to cross it off the list.",
    "You deserve the feeling of accomplishment. Start.",
    "The longer you wait, the harder it gets. Act now.",
    "Your future gratitude starts with one step right now.",
    "Do it for yourself. Not for a grade, not for a deadline.",
    "The task won't go away. Better deal with it now.",

    // ── Gentle humor ─────────────────────────────────────────────────────
    "Open a task. No, not the one with the cat videos.",
    "You know what you need to do. We both know.",
    "You can use your phone for more than scrolling. Try it.",
    "You've already read this message. Now open a task.",
    "Procrastination is a skill too. Maybe time to master productivity?",
    "Yes, this is a nudge. No, I won't leave until you start.",
    "You can dismiss this notification. Or close a task. Your choice.",
    "This nudge will disappear, but the task won't.",
    "Scrolling? The task is waiting. Patient, but not infinite.",
    "Come on, one more task. You're not the type to give up.",
];

/// Returns a nudge text for the given locale. `seed` should be derived from
/// per-fire entropy (e.g. job id) so two consecutive fires don't pick the
/// same line.
pub fn nudge_text(locale: Locale, seed: u64) -> &'static str {
    let bank = match locale {
        Locale::Ru => MESSAGES_RU,
        Locale::En => MESSAGES_EN,
    };
    rand_lite::pick(bank, seed).copied().unwrap_or(bank[0])
}

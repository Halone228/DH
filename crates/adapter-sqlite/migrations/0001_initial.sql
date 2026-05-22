CREATE TABLE users (
    id            TEXT PRIMARY KEY,
    telegram_id   INTEGER NOT NULL UNIQUE,
    username      TEXT,
    timezone      TEXT NOT NULL,
    locale        TEXT NOT NULL DEFAULT 'ru',
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE nudge_settings (
    user_id              TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    enabled              INTEGER NOT NULL DEFAULT 1,
    daily_count          INTEGER NOT NULL DEFAULT 5,
    active_window_start  TEXT NOT NULL DEFAULT '09:00:00',
    active_window_end    TEXT NOT NULL DEFAULT '21:00:00'
);

CREATE TABLE reminders (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    text            TEXT NOT NULL,
    recurrence_json TEXT NOT NULL,
    active          INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL
);

CREATE INDEX idx_reminders_user_active ON reminders(user_id, active);

CREATE TABLE scheduled_jobs (
    id           TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind         TEXT NOT NULL,                   -- 'reminder' | 'nudge'
    reminder_id  TEXT REFERENCES reminders(id) ON DELETE CASCADE,
    payload      TEXT,                            -- nudge message
    fire_at      TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    fired_at     TEXT,
    CHECK (kind IN ('reminder', 'nudge')),
    CHECK (
        (kind = 'reminder' AND reminder_id IS NOT NULL) OR
        (kind = 'nudge'    AND payload     IS NOT NULL)
    )
);

CREATE INDEX idx_jobs_due ON scheduled_jobs(fire_at) WHERE fired_at IS NULL;
CREATE INDEX idx_jobs_reminder ON scheduled_jobs(reminder_id) WHERE fired_at IS NULL;
CREATE INDEX idx_jobs_user_kind ON scheduled_jobs(user_id, kind) WHERE fired_at IS NULL;

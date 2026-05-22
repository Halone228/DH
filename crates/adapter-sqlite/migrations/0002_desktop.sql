CREATE TABLE desktop_tokens (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash    TEXT NOT NULL UNIQUE,        -- SHA-256 hex of bearer
    label         TEXT NOT NULL,
    created_at    TEXT NOT NULL,
    last_seen_at  TEXT,
    revoked_at    TEXT
);

CREATE INDEX idx_desktop_tokens_user ON desktop_tokens(user_id) WHERE revoked_at IS NULL;

CREATE TABLE desktop_activity (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    app_name      TEXT NOT NULL,
    window_title  TEXT,
    started_at    TEXT NOT NULL,
    ended_at      TEXT NOT NULL,
    received_at   TEXT NOT NULL
);

CREATE INDEX idx_desktop_activity_user_time ON desktop_activity(user_id, ended_at);

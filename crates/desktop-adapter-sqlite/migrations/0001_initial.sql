CREATE TABLE activity_events (
    id          TEXT PRIMARY KEY,
    app_name    TEXT NOT NULL,
    window_title TEXT,
    started_at  TEXT NOT NULL,
    ended_at    TEXT NOT NULL,
    synced      INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_activity_synced ON activity_events(synced, ended_at);

CREATE TABLE local_notifications (
    id           TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    body         TEXT NOT NULL,
    fire_at      TEXT NOT NULL,
    category     TEXT NOT NULL,
    state        TEXT NOT NULL,            -- 'pending' | 'fired' | 'skipped'
    fired_at     TEXT,
    ack_pending  INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_notif_pending ON local_notifications(state, fire_at);
CREATE INDEX idx_notif_acks ON local_notifications(ack_pending) WHERE ack_pending = 1;

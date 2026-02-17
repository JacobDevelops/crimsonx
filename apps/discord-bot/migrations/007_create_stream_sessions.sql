CREATE TABLE IF NOT EXISTS stream_sessions (
    id BIGSERIAL PRIMARY KEY,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    title TEXT,
    game TEXT,
    peak_viewers INTEGER NOT NULL DEFAULT 0,
    notification_message_id BIGINT
);

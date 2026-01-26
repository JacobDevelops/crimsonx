CREATE TABLE IF NOT EXISTS mod_actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    moderator_id INTEGER NOT NULL,
    action_type TEXT NOT NULL,
    reason TEXT,
    duration TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_mod_actions_user ON mod_actions (guild_id, user_id);

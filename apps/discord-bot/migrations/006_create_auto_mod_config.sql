CREATE TABLE IF NOT EXISTS auto_mod_config (
    guild_id INTEGER PRIMARY KEY NOT NULL,
    banned_words TEXT NOT NULL DEFAULT '[]',
    max_mentions INTEGER NOT NULL DEFAULT 5,
    spam_threshold INTEGER NOT NULL DEFAULT 5,
    link_allowlist TEXT NOT NULL DEFAULT '[]'
);

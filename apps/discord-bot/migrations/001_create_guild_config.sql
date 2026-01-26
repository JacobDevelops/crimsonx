CREATE TABLE IF NOT EXISTS guild_config (
    guild_id INTEGER PRIMARY KEY NOT NULL,
    prefix TEXT DEFAULT '!',
    welcome_channel_id INTEGER,
    log_channel_id INTEGER,
    live_channel_id INTEGER,
    live_role_id INTEGER,
    mod_role_id INTEGER
);

CREATE TABLE IF NOT EXISTS guild_config (
    guild_id BIGINT PRIMARY KEY NOT NULL,
    prefix TEXT DEFAULT '!',
    welcome_channel_id BIGINT,
    log_channel_id BIGINT,
    live_channel_id BIGINT,
    live_role_id BIGINT,
    mod_role_id BIGINT
);

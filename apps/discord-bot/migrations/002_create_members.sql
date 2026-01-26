CREATE TABLE IF NOT EXISTS members (
    guild_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    join_date TEXT,
    message_count INTEGER NOT NULL DEFAULT 0,
    xp INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 0,
    currency_balance INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (guild_id, user_id)
);

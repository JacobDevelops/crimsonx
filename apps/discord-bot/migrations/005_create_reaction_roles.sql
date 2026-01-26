CREATE TABLE IF NOT EXISTS reaction_roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL,
    message_id INTEGER NOT NULL,
    emoji TEXT NOT NULL,
    role_id INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reaction_roles_message ON reaction_roles (guild_id, message_id);

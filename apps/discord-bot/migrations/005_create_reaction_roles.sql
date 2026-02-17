CREATE TABLE IF NOT EXISTS reaction_roles (
    id BIGSERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    message_id BIGINT NOT NULL,
    emoji TEXT NOT NULL,
    role_id BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reaction_roles_message ON reaction_roles (guild_id, message_id);

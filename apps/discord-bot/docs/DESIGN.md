# CrimsonX Discord Bot - System Design Specification

## Overview

A comprehensive Discord bot built in Rust to fully manage a Twitch community Discord server. Designed for growth from a small community to a large, active server.

### Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Language | Rust | Performance, safety, async support |
| Discord Library | Serenity + Poise | Mature ecosystem, slash command support |
| Database | PostgreSQL | Relational data, excellent Rust support (sqlx) |
| Configuration | TOML files | Rust-native, version controllable |
| Async Runtime | Tokio | Industry standard for Rust async |

---

## Architecture

### High-Level System Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Discord API                              │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                       CrimsonX Bot Core                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Gateway   │  │   Command   │  │     Event Dispatcher    │  │
│  │   Handler   │──│   Router    │──│  (messages, reactions,  │  │
│  │  (Serenity) │  │   (Poise)   │  │   voice, members, etc)  │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
            ┌───────────────────┼───────────────────┐
            ▼                   ▼                   ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐
│    Modules      │  │    Services     │  │    Integrations     │
│  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────────┐  │
│  │  Levels   │  │  │  │  Database │  │  │  │  Twitch API   │  │
│  ├───────────┤  │  │  ├───────────┤  │  │  └───────────────┘  │
│  │Moderation │  │  │  │   Config  │  │  └─────────────────────┘
│  ├───────────┤  │  │  ├───────────┤  │
│  │Role Menus │  │  │  │  Logging  │  │
│  ├───────────┤  │  │  └───────────┘  │
│  │ Welcome   │  │  └─────────────────┘
│  ├───────────┤  │
│  │  Custom   │  │
│  │ Commands  │  │
│  └───────────┘  │
└─────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────┐
│                        PostgreSQL Database                       │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────────┐  │
│  │ members │ │ levels  │ │mod_logs │ │  roles  │ │  configs  │  │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └───────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Directory Structure

```
discord-bot/
├── Cargo.toml
├── config/
│   ├── config.toml              # Main configuration
│   └── config.example.toml      # Example configuration
├── migrations/                   # SQL migrations (sqlx)
│   ├── 001_initial_schema.sql
│   └── ...
├── src/
│   ├── main.rs                  # Entry point
│   ├── lib.rs                   # Library root
│   ├── bot.rs                   # Bot initialization
│   ├── config.rs                # Configuration loading
│   ├── error.rs                 # Error types
│   │
│   ├── commands/                # Slash commands (Poise)
│   │   ├── mod.rs
│   │   ├── levels.rs            # /level, /leaderboard, /rank
│   │   ├── moderation.rs        # /kick, /ban, /warn, /mute
│   │   ├── roles.rs             # /rolemenu create, /autorole
│   │   ├── custom.rs            # /command add, /command remove
│   │   ├── admin.rs             # /config, /settings
│   │   └── twitch.rs            # /twitch notify, /twitch live
│   │
│   ├── events/                  # Event handlers
│   │   ├── mod.rs
│   │   ├── message.rs           # Message events (XP, custom cmds)
│   │   ├── reaction.rs          # Reaction events (XP, role menus)
│   │   ├── member.rs            # Join/leave events
│   │   ├── voice.rs             # Voice state changes (XP)
│   │   └── interaction.rs       # Button/select menu interactions
│   │
│   ├── modules/                 # Feature modules
│   │   ├── mod.rs
│   │   ├── levels/
│   │   │   ├── mod.rs
│   │   │   ├── xp.rs            # XP calculation & storage
│   │   │   ├── rewards.rs       # Level-up rewards (future)
│   │   │   └── leaderboard.rs   # Leaderboard generation
│   │   ├── moderation/
│   │   │   ├── mod.rs
│   │   │   ├── actions.rs       # Kick, ban, mute, warn
│   │   │   ├── automod.rs       # Automated moderation (future)
│   │   │   └── logging.rs       # Mod action logging
│   │   ├── roles/
│   │   │   ├── mod.rs
│   │   │   ├── menu.rs          # Role menu creation/handling
│   │   │   └── autorole.rs      # Auto-assign on join
│   │   ├── welcome/
│   │   │   ├── mod.rs
│   │   │   └── messages.rs      # Welcome/leave messages
│   │   ├── custom_commands/
│   │   │   ├── mod.rs
│   │   │   └── handler.rs       # Custom command execution
│   │   └── twitch/
│   │       ├── mod.rs
│   │       ├── api.rs           # Twitch API client
│   │       └── poller.rs        # Stream polling & notifications
│   │
│   ├── services/                # Shared services
│   │   ├── mod.rs
│   │   ├── database.rs          # Database connection pool
│   │   └── cache.rs             # In-memory caching
│   │
│   └── models/                  # Data models
│       ├── mod.rs
│       ├── member.rs
│       ├── level.rs
│       ├── mod_action.rs
│       ├── role_menu.rs
│       ├── custom_command.rs
│       └── guild_config.rs
└── docs/
    └── DESIGN.md                # This file
```

---

## Database Schema

### Entity Relationship Diagram

```
┌─────────────────┐       ┌─────────────────┐
│  guild_configs  │       │     members     │
├─────────────────┤       ├─────────────────┤
│ guild_id (PK)   │───┐   │ id (PK)         │
│ welcome_channel │   │   │ guild_id (FK)   │──┐
│ leave_channel   │   │   │ user_id         │  │
│ mod_log_channel │   │   │ joined_at       │  │
│ level_channel   │   └──▶│ ...             │  │
│ prefix          │       └─────────────────┘  │
│ ...             │                            │
└─────────────────┘       ┌─────────────────┐  │
                          │  member_levels  │  │
┌─────────────────┐       ├─────────────────┤  │
│   role_menus    │       │ id (PK)         │  │
├─────────────────┤       │ member_id (FK)  │◀─┤
│ id (PK)         │       │ xp              │  │
│ guild_id        │       │ level           │  │
│ channel_id      │       │ messages_count  │  │
│ message_id      │       │ reactions_count │  │
│ title           │       │ voice_minutes   │  │
│ roles (jsonb)   │       │ last_xp_at      │  │
└─────────────────┘       └─────────────────┘  │
                                               │
┌─────────────────┐       ┌─────────────────┐  │
│   auto_roles    │       │   mod_actions   │  │
├─────────────────┤       ├─────────────────┤  │
│ id (PK)         │       │ id (PK)         │  │
│ guild_id        │       │ guild_id        │  │
│ role_id         │       │ target_id (FK)  │◀─┤
│ trigger         │       │ moderator_id    │  │
│ ...             │       │ action_type     │  │
└─────────────────┘       │ reason          │  │
                          │ created_at      │  │
┌─────────────────┐       │ expires_at      │  │
│ custom_commands │       └─────────────────┘  │
├─────────────────┤                            │
│ id (PK)         │       ┌─────────────────┐  │
│ guild_id        │       │   warnings      │  │
│ name            │       ├─────────────────┤  │
│ response        │       │ id (PK)         │  │
│ embed (jsonb)   │       │ member_id (FK)  │◀─┘
│ created_by      │       │ moderator_id    │
│ uses_count      │       │ reason          │
└─────────────────┘       │ created_at      │
                          └─────────────────┘

┌─────────────────────┐
│  twitch_config      │
├─────────────────────┤
│ guild_id (PK)       │
│ twitch_id           │
│ twitch_login        │
│ notify_channel_id   │
│ notify_template     │
│ live_role_id        │
│ last_live_at        │
└─────────────────────┘
```

### SQL Schema

```sql
-- Guild configuration
CREATE TABLE guild_configs (
    guild_id BIGINT PRIMARY KEY,
    welcome_channel_id BIGINT,
    welcome_message TEXT,
    leave_channel_id BIGINT,
    leave_message TEXT,
    mod_log_channel_id BIGINT,
    level_announce_channel_id BIGINT,
    xp_cooldown_seconds INT DEFAULT 60,
    xp_per_message INT DEFAULT 15,
    xp_per_reaction INT DEFAULT 5,
    xp_per_voice_minute INT DEFAULT 10,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Members (guild-specific)
CREATE TABLE members (
    id BIGSERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(guild_id, user_id)
);

-- Member levels and XP
CREATE TABLE member_levels (
    id BIGSERIAL PRIMARY KEY,
    member_id BIGINT REFERENCES members(id) ON DELETE CASCADE,
    xp BIGINT DEFAULT 0,
    level INT DEFAULT 0,
    messages_count BIGINT DEFAULT 0,
    reactions_count BIGINT DEFAULT 0,
    voice_minutes BIGINT DEFAULT 0,
    last_xp_at TIMESTAMPTZ,
    UNIQUE(member_id)
);

-- Role menus
CREATE TABLE role_menus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    message_id BIGINT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    roles JSONB NOT NULL, -- [{role_id, emoji, label, description}]
    max_selections INT DEFAULT 0, -- 0 = unlimited
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Auto-assign roles
CREATE TABLE auto_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    trigger TEXT NOT NULL, -- 'join', 'boost', etc.
    UNIQUE(guild_id, role_id, trigger)
);

-- Custom commands
CREATE TABLE custom_commands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id BIGINT NOT NULL,
    name VARCHAR(32) NOT NULL,
    response TEXT,
    embed JSONB,
    created_by BIGINT NOT NULL,
    uses_count BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(guild_id, name)
);

-- Moderation actions
CREATE TABLE mod_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id BIGINT NOT NULL,
    target_user_id BIGINT NOT NULL,
    moderator_user_id BIGINT NOT NULL,
    action_type VARCHAR(16) NOT NULL, -- 'warn', 'mute', 'kick', 'ban'
    reason TEXT,
    duration_seconds BIGINT, -- NULL for permanent/instant
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

-- Twitch integration (single channel per guild)
CREATE TABLE twitch_config (
    guild_id BIGINT PRIMARY KEY,
    twitch_id VARCHAR(32) NOT NULL,
    twitch_login VARCHAR(64) NOT NULL,
    notify_channel_id BIGINT,
    notify_template TEXT,
    live_role_id BIGINT,
    last_live_at TIMESTAMPTZ
);

-- Indexes for performance
CREATE INDEX idx_members_guild ON members(guild_id);
CREATE INDEX idx_member_levels_xp ON member_levels(xp DESC);
CREATE INDEX idx_mod_actions_target ON mod_actions(guild_id, target_user_id);
CREATE INDEX idx_mod_actions_expires ON mod_actions(expires_at) WHERE expires_at IS NOT NULL;
```

---

## Module Specifications

### 1. Leveling System

#### XP Sources and Values (Configurable per-guild)

| Activity | Base XP | Cooldown | Notes |
|----------|---------|----------|-------|
| Message | 15-25 | 60s | Randomized within range |
| Reaction (given) | 5 | 30s | Per unique message |
| Reaction (received) | 3 | None | Passive XP gain |
| Voice (per minute) | 10 | None | Must be unmuted, in non-AFK channel |
| Event participation | 50 | Per event | Future: integration with events |

#### Level Curve

```
XP required for level N = 5 * (N^2) + 50 * N + 100

Level 1:  155 XP
Level 5:  475 XP
Level 10: 1,100 XP
Level 20: 3,100 XP
Level 50: 15,100 XP
Level 100: 55,100 XP
```

#### Commands

| Command | Description | Permissions |
|---------|-------------|-------------|
| `/rank [user]` | Show level, XP, rank position | Everyone |
| `/leaderboard [page]` | Top members by XP | Everyone |
| `/xp give <user> <amount>` | Grant XP to user | Manage Server |
| `/xp remove <user> <amount>` | Remove XP from user | Manage Server |
| `/xp reset <user>` | Reset user's XP to 0 | Administrator |

#### Rank Card Design (Embed-based)

Uses Discord embeds with emoji progress bars for simplicity:

```
┌─────────────────────────────────────────────────────┐
│  📊 Level 15                           Rank #3      │
├─────────────────────────────────────────────────────┤
│  ██████████████░░░░░░░░░░░░                         │
│  2,450 / 3,100 XP (79%)                             │
│                                                      │
│  📝 Messages: 1,234                                  │
│  👍 Reactions: 567                                   │
│  🎤 Voice: 89 hours                                  │
└─────────────────────────────────────────────────────┘
```

Progress bar uses custom emojis or Unicode blocks:
- `▓▓▓▓▓▓▓▓░░░░` (block characters)
- Or custom emojis: `:progress_full::progress_full::progress_empty:`

---

### 2. Moderation System

#### Actions

| Action | Command | Effect | Logging |
|--------|---------|--------|---------|
| Warn | `/warn <user> <reason>` | Record warning, DM user | Yes |
| Mute | `/mute <user> <duration> [reason]` | Timeout user | Yes |
| Kick | `/kick <user> [reason]` | Remove from server | Yes |
| Ban | `/ban <user> [duration] [reason]` | Ban from server | Yes |
| Unban | `/unban <user>` | Remove ban | Yes |
| Clear | `/clear <count> [user]` | Bulk delete messages | Yes |

#### Mod Log Format

```
┌─────────────────────────────────────────────────┐
│ 🔨 WARN                          Case #127      │
├─────────────────────────────────────────────────┤
│ Member: @User#0000 (123456789)                  │
│ Moderator: @Mod#0000                            │
│ Reason: Spamming in general chat                │
│ Date: Jan 26, 2026 at 3:45 PM                   │
└─────────────────────────────────────────────────┘
```

#### Commands (Additional)

| Command | Description | Permissions |
|---------|-------------|-------------|
| `/case <id>` | View case details | Manage Messages |
| `/history <user>` | View user's mod history | Manage Messages |
| `/reason <case_id> <reason>` | Update case reason | Manage Messages |

---

### 3. Role Menus

#### Creation Flow

```
/rolemenu create
  → Modal: Title, Description
  → Select: Channel to post in
  → Button: Add Role (repeatable)
    → Select: Role
    → Input: Emoji, Description
  → Select: Style (buttons/dropdown)
  → Select: Max selections (0 = unlimited)
  → Confirm & Post
```

#### Role Menu Styles

**Button Style:**
```
┌─────────────────────────────────────────────────┐
│  🎨 Color Roles                                  │
│  Select your preferred name color               │
├─────────────────────────────────────────────────┤
│  [🔴 Red] [🟠 Orange] [🟡 Yellow]               │
│  [🟢 Green] [🔵 Blue] [🟣 Purple]               │
└─────────────────────────────────────────────────┘
```

**Dropdown Style:**
```
┌─────────────────────────────────────────────────┐
│  🔔 Notification Roles                          │
│  Select which pings you want to receive         │
├─────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────┐        │
│  │ Select roles...                   ▼ │        │
│  └─────────────────────────────────────┘        │
└─────────────────────────────────────────────────┘
```

---

### 4. Auto-Role System

#### Triggers

| Trigger | Description | Configuration |
|---------|-------------|---------------|
| `join` | Assign when member joins | Role ID |
| `boost` | Assign when member boosts | Role ID |
| `verify` | Assign after verification (future) | Role ID + method |

#### Commands

| Command | Description | Permissions |
|---------|-------------|-------------|
| `/autorole add <trigger> <role>` | Add auto-role | Manage Roles |
| `/autorole remove <trigger> <role>` | Remove auto-role | Manage Roles |
| `/autorole list` | List all auto-roles | Manage Roles |

---

### 5. Welcome/Leave System

#### Template Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `{user}` | User mention | @Username |
| `{user.name}` | Username | Username |
| `{user.tag}` | Full tag | Username#0000 |
| `{server}` | Server name | My Server |
| `{member_count}` | Total members | 150 |
| `{member_count_ord}` | Ordinal | 150th |

#### Configuration Commands

| Command | Description |
|---------|-------------|
| `/welcome channel <channel>` | Set welcome channel |
| `/welcome message <text>` | Set welcome message |
| `/welcome embed` | Configure embed (opens modal) |
| `/welcome test` | Preview welcome message |
| `/leave channel <channel>` | Set leave channel |
| `/leave message <text>` | Set leave message |

---

### 6. Custom Commands

#### Command Types

| Type | Description | Example |
|------|-------------|---------|
| Text | Simple text response | `/command add rules "Check #rules"` |
| Embed | Rich embed response | `/command embed add info` (opens modal) |

#### Commands

| Command | Description | Permissions |
|---------|-------------|-------------|
| `/command add <name> <response>` | Create text command | Manage Server |
| `/command embed add <name>` | Create embed command | Manage Server |
| `/command remove <name>` | Delete command | Manage Server |
| `/command list` | List all commands | Everyone |
| `/command info <name>` | Show command details | Everyone |

---

### 7. Twitch Integration (Single Channel)

Simplified integration for your Twitch channel only.

#### Features

1. **Stream Notifications**: Post when you go live
2. **Live Role**: Assign role to you while streaming
3. **Channel Updates**: Update Discord channel name/topic with stream info

#### Configuration

```
/twitch setup
  → Opens modal for Twitch username
  → Verifies channel exists via API
  → Stores in config

/twitch notify <channel> [message]
  → Sets notification channel
  → Optional custom message template

/twitch liverole <role>
  → Assigns role when you're live
  → Removes role when offline

/twitch status
  → Shows current configuration
  → Shows if currently live
```

#### Notification Template Variables

| Variable | Description |
|----------|-------------|
| `{streamer}` | Your display name |
| `{title}` | Stream title |
| `{game}` | Game/category being played |
| `{url}` | Stream URL |
| `{viewers}` | Current viewer count |

#### Polling Strategy

- Poll Twitch API every 60 seconds for your channel
- Simple single-channel polling (no batching needed)
- Future: EventSub webhooks for instant notifications

---

## Configuration

### config.toml

```toml
[bot]
token = "${DISCORD_TOKEN}"  # Environment variable substitution
application_id = 123456789

[database]
url = "${DATABASE_URL}"
max_connections = 10

[twitch]
client_id = "${TWITCH_CLIENT_ID}"
client_secret = "${TWITCH_CLIENT_SECRET}"
poll_interval_seconds = 60

[logging]
level = "info"  # trace, debug, info, warn, error
format = "pretty"  # pretty, json

[features]
levels = true
moderation = true
role_menus = true
welcome = true
custom_commands = true
twitch = true
```

---

## API Rate Limit Handling

### Discord Rate Limits

```rust
// Poise/Serenity handle most rate limiting automatically
// For bulk operations, implement custom backoff:

const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 1000;

async fn with_retry<T, F, Fut>(f: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    for attempt in 0..MAX_RETRIES {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if is_rate_limit(&e) => {
                let delay = BASE_DELAY_MS * 2u64.pow(attempt);
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(Error::MaxRetriesExceeded)
}
```

### Twitch Rate Limits

- 800 requests per minute for authenticated requests
- Batch channel status checks (up to 100 per request)
- Implement token bucket rate limiter

---

## Error Handling Strategy

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Discord API error: {0}")]
    Discord(#[from] serenity::Error),

    #[error("Twitch API error: {0}")]
    Twitch(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

### User-Facing Error Messages

- Never expose internal error details
- Provide actionable feedback
- Log full error details for debugging

```rust
impl Error {
    pub fn user_message(&self) -> &str {
        match self {
            Error::Database(_) => "A database error occurred. Please try again later.",
            Error::Discord(_) => "Failed to communicate with Discord. Please try again.",
            Error::PermissionDenied(msg) => msg,
            Error::NotFound(msg) => msg,
            Error::InvalidInput(msg) => msg,
            _ => "An unexpected error occurred.",
        }
    }
}
```

---

## Security Considerations

### Permission Checks

```rust
// All mod commands verify permissions before execution
async fn check_mod_permissions(ctx: Context<'_>) -> Result<bool> {
    let member = ctx.author_member().await?;
    let permissions = member.permissions(ctx)?;

    Ok(permissions.kick_members() || permissions.ban_members())
}
```

### Input Validation

- Sanitize all user input before database storage
- Validate role/channel IDs exist and bot has access
- Prevent command injection in custom commands
- Rate limit command usage per user

### Token Security

- Never log tokens or secrets
- Use environment variables for sensitive data
- Implement token refresh for Twitch OAuth

---

## Deployment

### Docker

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/discord-bot /usr/local/bin/
COPY config/ /etc/discord-bot/
CMD ["discord-bot"]
```

### Environment Variables

```env
DISCORD_TOKEN=your_bot_token
DATABASE_URL=postgres://user:pass@localhost/crimsonx
TWITCH_CLIENT_ID=your_twitch_client_id
TWITCH_CLIENT_SECRET=your_twitch_client_secret
RUST_LOG=discord_bot=info
```

---

## Development Phases

### Phase 1: Core Foundation
- [ ] Project setup with Serenity + Poise
- [ ] Database connection and migrations
- [ ] Configuration loading
- [ ] Basic command structure
- [ ] Error handling framework

### Phase 2: Leveling System
- [ ] XP tracking (messages, reactions, voice)
- [ ] Level calculation and storage
- [ ] `/rank` command with card generation
- [ ] `/leaderboard` command
- [ ] Level-up announcements

### Phase 3: Moderation
- [ ] Warn, mute, kick, ban commands
- [ ] Mod log channel
- [ ] Case tracking
- [ ] User history

### Phase 4: Role Management
- [ ] Role menu creation
- [ ] Button/dropdown interactions
- [ ] Auto-role on join
- [ ] Role persistence

### Phase 5: Welcome System
- [ ] Welcome messages
- [ ] Leave messages
- [ ] Template variable parsing
- [ ] Embed support

### Phase 6: Custom Commands
- [ ] Text command creation
- [ ] Embed command creation
- [ ] Command listing and info
- [ ] Usage tracking

### Phase 7: Twitch Integration
- [ ] Twitch API client
- [ ] Stream polling
- [ ] Live notifications
- [ ] Live role assignment

### Phase 8: Polish
- [ ] Error message improvements
- [ ] Help command
- [ ] Admin dashboard (future)
- [ ] Analytics and insights

---

## Dependencies

```toml
[dependencies]
# Discord
serenity = { version = "0.12", features = ["framework", "gateway", "cache"] }
poise = "0.6"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }

# Configuration
config = "0.14"
toml = "0.8"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
thiserror = "1"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# HTTP client (for Twitch)
reqwest = { version = "0.12", features = ["json"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
dashmap = "5"  # Concurrent hashmap for caching
```

---

## Design Decisions Summary

Based on your input:

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Rank cards | Embed-based | Simpler implementation, no image dependencies |
| Auto-moderation | Deferred | Start with manual mod, add automod later |
| Twitch scope | Single channel | Your channel only, simplified configuration |

---

## Next Steps

After reviewing and approving this design:

1. Run `/sc:implement` to set up the project structure
2. Start with Phase 1 (Core Foundation)
3. Iterate through phases, testing each feature

The design is intentionally modular - each feature can be developed and tested independently.

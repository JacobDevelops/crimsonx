# CrimsonBot Implementation Workflow

> Generated from `crimsonbot-spec.docx.pdf` (v1.0, February 2026)
> Date: 2026-02-17
> Strategy: Systematic | Depth: Deep

---

## Current State Analysis

### What Exists (Phase 1 - Partial)
- **Bot connects** to Discord via Serenity `EventHandler` (raw, no Poise)
- **Auto-role** on `guild_member_addition` via `AUTOROLE_IDS` env var
- **Config** loading from `.env` (token + autorole IDs only)
- **Error types** with `thiserror` (Discord, Config variants)
- **Logging** via `tracing` + `tracing-subscriber` with env filter
- **Deployment** pipeline: Dockerfile + Railway.app config
- **Feature flags** in `config/config.toml` (all disabled, not wired up)

### What's Missing (Gap Analysis)
| Area | Gap |
|------|-----|
| **Framework** | No Poise integration - all commands need Poise slash command framework |
| **Database** | No SQLite/sqlx - needed for members, warnings, roles, economy |
| **Project structure** | Flat `src/` - spec requires `commands/`, `events/`, `integrations/`, `utils/` modules |
| **Welcome system** | No welcome embeds, no leave logging, no mod-logs |
| **Info commands** | No `/ping`, `/help`, `/about`, `/socials`, `/schedule`, `/server` |
| **Bot lifecycle** | No status setting, no SIGINT/SIGTERM handling, no graceful shutdown |
| **Embed theming** | No shared embed builder with CrimsonX branding (#DC143C) |
| **Twitch** | No EventSub, no go-live notifications, no Helix API |
| **Roles** | No reaction/button role system, no `/role` commands |
| **Moderation** | No warn/kick/ban/timeout/purge, no auto-mod, no message logging |
| **GitHub** | No webhook receiver, no `/repo`/`/issues`/`/commits` commands |
| **Economy** | No currency system, no leaderboards, no mini-games |
| **Advanced** | No Twitch chat bridge, no clip system, no StreamElements |

---

## Implementation Phases

Each phase maps to 1-2 dev stream episodes. Phases are sequential with internal tasks parallelizable where noted.

---

## Phase 1: Core Foundation (Episode 1-2)

> Rebuild the bot skeleton on Poise, add database, restructure project, implement welcome system and info commands.

### 1.1 — Project Restructure & Poise Migration

**Priority**: CRITICAL (blocks everything)
**Branch**: `feat/poise-migration`

#### Tasks

- [ ] **1.1.1** Add Poise + sqlx + supporting crates to `Cargo.toml`
  ```
  poise = "0.6"
  sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
  reqwest = { version = "0.12", features = ["json"] }
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  chrono = "0.4"
  rand = "0.8"
  ```

- [ ] **1.1.2** Create module directory structure per spec:
  ```
  src/
  ├── main.rs                  # Entry point, Poise framework setup
  ├── lib.rs                   # Re-exports
  ├── config.rs                # Expanded config (database_url, channel IDs, etc.)
  ├── db.rs                    # SQLite pool setup, migrations
  ├── error.rs                 # Expanded error enum (+ Database variant)
  ├── commands/
  │   ├── mod.rs
  │   └── general.rs           # /ping, /help, /about, /socials, /server, /schedule
  ├── events/
  │   ├── mod.rs
  │   └── member.rs            # Join/leave handlers (welcome + auto-role)
  ├── integrations/
  │   └── mod.rs               # Placeholder for Phase 2+
  └── utils/
      ├── mod.rs
      ├── embeds.rs            # Themed embed builder
      └── permissions.rs       # Permission check helpers
  ```

- [ ] **1.1.3** Define `Data` struct (Poise user data) holding:
  - `db: sqlx::SqlitePool`
  - `config: Config`
  - `start_time: std::time::Instant` (for uptime in `/about`)

- [ ] **1.1.4** Rewrite `main.rs` to use `poise::Framework`:
  - Build framework with `poise::Framework::builder()`
  - Register all commands in `setup` closure
  - Set gateway intents per spec (add GUILDS, GUILD_MESSAGE_REACTIONS, GUILD_VOICE_STATES)
  - Set `event_handler` for non-command events (member join/leave)
  - Set default bot activity status: "Watching the crimson tide"

- [ ] **1.1.5** Migrate auto-role logic from raw `EventHandler` to Poise `event_handler`

**Checkpoint**: Bot compiles, connects via Poise, auto-role still works.

---

### 1.2 — Database Layer

**Priority**: HIGH (blocks moderation, economy, roles)
**Depends on**: 1.1

#### Tasks

- [ ] **1.2.1** Create `src/db.rs`:
  - `init_pool(database_url: &str) -> SqlitePool`
  - Run embedded migrations via `sqlx::migrate!()`

- [ ] **1.2.2** Create `migrations/` directory with initial schema:
  - `001_create_guild_config.sql` — guild_id (PK), prefix, welcome_channel_id, log_channel_id, live_channel_id, live_role_id, mod_role_id
  - `002_create_members.sql` — guild_id + user_id (composite PK), join_date, message_count, xp, level, currency_balance
  - `003_create_warnings.sql` — id (PK), guild_id, user_id, moderator_id, reason, created_at
  - `004_create_mod_actions.sql` — id (PK), guild_id, user_id, moderator_id, action_type, reason, duration, created_at
  - `005_create_reaction_roles.sql` — id (PK), guild_id, message_id, emoji, role_id
  - `006_create_auto_mod_config.sql` — guild_id (PK), banned_words (JSON), max_mentions, spam_threshold, link_allowlist (JSON)

- [ ] **1.2.3** Add `DATABASE_URL` to `Config` (required, default: `sqlite:crimsonbot.db`)

- [ ] **1.2.4** Update `.env.example` with `DATABASE_URL=sqlite:crimsonbot.db`

**Checkpoint**: Database initializes on startup, migrations run, tables created.

---

### 1.3 — Embed Theme System

**Priority**: HIGH (used by every user-facing feature)
**Depends on**: 1.1

#### Tasks

- [ ] **1.3.1** Implement `src/utils/embeds.rs`:
  - `CrimsonEmbed` builder struct wrapping `serenity::CreateEmbed`
  - Default color: `#DC143C` (crimson)
  - Default footer: "CrimsonX • 0xDC143C" with bot avatar
  - Always include UTC timestamp
  - Color variants: Success (#00FF7F), Warning (#FFD700), Error (#FF4444), Moderation (#8B0A1E), Twitch (#9146FF), GitHub (#238636), Economy (#00CED1)
  - Helper methods: `success()`, `error()`, `warning()`, `moderation()`, `twitch()`, `github()`

**Checkpoint**: Embed builders produce correctly themed embeds.

---

### 1.4 — Welcome System

**Priority**: MEDIUM
**Depends on**: 1.1, 1.2, 1.3

#### Tasks

- [ ] **1.4.1** Implement `src/events/member.rs` — `guild_member_addition`:
  - Auto-role assignment (migrated from current code)
  - Send themed welcome embed to `#welcome` channel (configurable via guild_config)
  - Embed includes: member name, avatar as author icon, server logo thumbnail, randomized welcome message, mention #rules and #roles
  - Log join event to `#mod-logs` with timestamp and account age

- [ ] **1.4.2** Implement `guild_member_removal` handler:
  - Log leave event to `#mod-logs` with roles held and join duration

- [ ] **1.4.3** Add configurable welcome messages list to `config/config.toml`:
  ```toml
  [welcome]
  channel_id = 0
  messages = [
    "Welcome to The Crimson Den, {user}!",
    "{user} just joined the crew!",
  ]
  ```

**Checkpoint**: New members get welcome embed + auto-role. Leaves are logged.

---

### 1.5 — Information Commands

**Priority**: MEDIUM
**Depends on**: 1.1, 1.3

#### Tasks

- [ ] **1.5.1** `/ping` — Measure gateway + API latency in ms
- [ ] **1.5.2** `/about` — Bot version, uptime, GitHub repo link embed
- [ ] **1.5.3** `/socials` — Embed with Twitch, YouTube, X, GitHub links (from config)
- [ ] **1.5.4** `/schedule` — Embed with streaming schedule (from config/config.toml)
- [ ] **1.5.5** `/server` — Embed with member count, creation date, boost level
- [ ] **1.5.6** `/help` — Paginated embed grouped by command category

**Checkpoint**: All 6 info commands register as slash commands and respond with themed embeds.

---

### Phase 1 Validation
- [ ] Bot connects and sets "Watching the crimson tide" status
- [ ] Auto-role assigns on member join
- [ ] Welcome embed posts to #welcome
- [ ] Join/leave logged to #mod-logs
- [ ] All 6 info commands work as slash commands
- [ ] Database initializes with all tables
- [ ] SIGINT/SIGTERM triggers clean shutdown log
- [ ] `cargo test` passes (unit tests for config, embeds)
- [ ] `cargo clippy` clean

---

## Phase 2: Twitch Stream Integration (Episode 3)

> Connect to Twitch EventSub via WebSocket for go-live/offline notifications.

**Branch**: `feat/twitch-integration`
**Depends on**: Phase 1 complete

### 2.1 — Twitch Authentication

- [ ] **2.1.1** Add env vars to config: `TWITCH_CLIENT_ID`, `TWITCH_CLIENT_SECRET`, `TWITCH_CHANNEL_ID`
- [ ] **2.1.2** Implement OAuth2 client credentials flow for app access token (reqwest)
- [ ] **2.1.3** Implement token refresh logic (refresh before expiry)
- [ ] **2.1.4** Create `src/integrations/twitch.rs` module

### 2.2 — EventSub WebSocket

- [ ] **2.2.1** Connect to Twitch EventSub WebSocket transport
- [ ] **2.2.2** Subscribe to events: `stream.online`, `stream.offline`, `channel.update`
- [ ] **2.2.3** Handle WebSocket keepalive and reconnection
- [ ] **2.2.4** Run EventSub listener as a Tokio background task alongside the Discord client

### 2.3 — Go-Live Notification Flow

- [ ] **2.3.1** On `stream.online`:
  - Fetch stream details from Helix API (title, game, thumbnail, viewer count)
  - Build crimson-themed embed with stream info + Twitch link
  - Ping the Live notification role
  - Post embed in `#go-live`
  - Store message ID in memory/db for later editing
  - Unlock `#live-chat` (remove send-message deny for @everyone)
  - Update bot status to "LIVE: [stream title]"

- [ ] **2.3.2** On `stream.offline`:
  - Edit go-live embed to show "STREAM ENDED" with duration + peak viewers
  - Lock `#live-chat`
  - Reset bot status to default

- [ ] **2.3.3** On `channel.update`:
  - Update the current go-live embed in real-time (title/game change)

- [ ] **2.3.4** Add `stream_sessions` table usage for stats tracking

### Phase 2 Validation
- [ ] Bot connects to Twitch EventSub on startup
- [ ] Go-live embed posts when stream starts
- [ ] Embed updates on title/game change
- [ ] Embed edits to "STREAM ENDED" when stream ends
- [ ] `#live-chat` locks/unlocks correctly
- [ ] Bot status changes with stream state
- [ ] Token refreshes automatically before expiry
- [ ] Graceful reconnection on WebSocket disconnect

---

## Phase 3: Role Management (Episode 4)

> Self-assignable roles via reaction embeds or button menus.

**Branch**: `feat/role-management`
**Depends on**: Phase 1 complete (database needed)

### 3.1 — Button Role System (Preferred per spec)

- [ ] **3.1.1** Create `src/commands/roles.rs` with subcommands:
  - `/role setup` (Admin) — Post role selection embed with buttons in #roles
  - `/role add` (Admin) — Add a new self-assignable role mapping
  - `/role remove` (Admin) — Remove a role mapping
  - `/role list` (Everyone) — Show all available self-assignable roles
  - `/role refresh` (Admin) — Re-post the role embed after config changes

- [ ] **3.1.2** Implement button interaction handler in `src/events/reaction.rs`:
  - On button click: toggle role (add if missing, remove if present)
  - Send ephemeral confirmation message
  - Buttons survive bot restarts (tied to message, not memory)

- [ ] **3.1.3** Store role mappings in `reaction_roles` table

- [ ] **3.1.4** Default role categories from spec:
  - Minecraft, Soulsborne, RPG, Dev, Live, Announcements, Content

### 3.2 — Reaction Role Fallback (Optional)

- [ ] **3.2.1** Monitor `reaction_add` / `reaction_remove` events
- [ ] **3.2.2** Map emoji to roles for tracked messages from `reaction_roles` table

### Phase 3 Validation
- [ ] `/role setup` posts button embed in #roles
- [ ] Clicking buttons toggles roles correctly
- [ ] Buttons work after bot restart
- [ ] `/role add` and `/role remove` persist to database
- [ ] `/role list` shows all available roles
- [ ] Permission checks enforce Admin-only on management commands

---

## Phase 4: Moderation (Episode 5-6)

> Full moderation suite with commands, auto-mod, and audit logging.

**Branch**: `feat/moderation`
**Depends on**: Phase 1 complete (database needed)

### 4.1 — Moderation Commands

- [ ] **4.1.1** Create `src/commands/moderation.rs`:
  - `/warn @user [reason]` (Mod) — Store warning in DB, DM user
  - `/timeout @user [duration] [reason]` (Mod) — Timeout via Discord API
  - `/kick @user [reason]` (Mod) — Kick from server
  - `/ban @user [reason] [delete_days]` (Mod) — Ban, optionally delete messages
  - `/unban @user` (Mod) — Unban by ID or username
  - `/warnings @user` (Mod) — View warning history from DB
  - `/clearwarnings @user` (Admin) — Clear all warnings
  - `/purge [count] [user]` (Mod) — Bulk delete up to 100 messages
  - `/slowmode [seconds]` (Mod) — Set channel slowmode (0 to disable)
  - `/lock [channel]` (Mod) — Lock channel
  - `/unlock [channel]` (Mod) — Unlock channel

- [ ] **4.1.2** All mod commands log action to `#mod-logs` with themed moderation embed
- [ ] **4.1.3** All mod commands store action in `mod_actions` table
- [ ] **4.1.4** Set `required_permissions` on all commands via Poise

### 4.2 — Auto-Moderation

- [ ] **4.2.1** Create `src/events/message.rs` — process every message:
  - **Banned words filter**: Check against `auto_mod_config.banned_words`, delete + warn via DM + log
  - **Spam detection**: Track message timestamps per user, flag 5+ msgs in 3 seconds, auto-timeout repeat offenders
  - **Link filter**: Block Discord invite links from non-mods, optional allowlist/blocklist
  - **Mention spam**: Auto-delete messages with 5+ unique mentions
  - **Duplicate message detection**: Detect identical messages in quick succession
  - **New account filter**: Flag accounts younger than 7 days (configurable), log to #mod-logs

- [ ] **4.2.2** Store auto-mod config per guild in `auto_mod_config` table

### 4.3 — Message Logging

- [ ] **4.3.1** Log message edits (before/after content, author, channel, timestamp)
- [ ] **4.3.2** Log message deletes (cached content if available, author, channel, timestamp)
- [ ] **4.3.3** Log role changes (user, roles added/removed, who made the change)
- [ ] **4.3.4** All logs post to `#mod-logs` as rich embeds

### Phase 4 Validation
- [ ] All 11 moderation commands work with proper permissions
- [ ] Warnings persist in database and show in `/warnings`
- [ ] Auto-mod catches banned words, spam, excessive mentions
- [ ] Message edits/deletes are logged
- [ ] All mod actions logged to #mod-logs with moderator attribution
- [ ] DMs sent to warned/timed-out users

---

## Phase 5: GitHub Integration (Episode 7)

> Webhook receiver for repo activity notifications + info commands.

**Branch**: `feat/github-integration`
**Depends on**: Phase 1 complete

### 5.1 — Webhook HTTP Server

- [ ] **5.1.1** Add `axum` dependency to Cargo.toml
- [ ] **5.1.2** Create `src/integrations/github.rs`:
  - Lightweight axum HTTP server running alongside Discord client (shared Tokio runtime)
  - `POST /github/webhook` endpoint
  - HMAC-SHA256 signature verification using `GITHUB_WEBHOOK_SECRET`
  - Parse `X-GitHub-Event` header to route events

- [ ] **5.1.3** Handle GitHub events:
  - `push` → Post to `#dev-announcements` (commit summary, author, file count, diff link)
  - `pull_request` (opened/merged) → Post to `#dev-announcements`
  - `issues` (opened/closed) → Post to `#dev-announcements`
  - `release` (published) → Post to `#announcements` (release name, changelog, download link)
  - `star` (created) → Optional fun notification to `#dev-chat`

### 5.2 — GitHub Commands

- [ ] **5.2.1** Create `src/commands/github.rs`:
  - `/repo` — Embed with repo URL, stars, open issues, last commit
  - `/issues [count]` — Paginated embed of open GitHub issues
  - `/commits [count]` — Embed with last N commits (default 5)
  - `/project` — Current dev stream project info (description, tech stack, progress)

- [ ] **5.2.2** Add `GITHUB_TOKEN` and `GITHUB_WEBHOOK_SECRET` to config
- [ ] **5.2.3** Use reqwest for GitHub API calls (with token for rate limits)

### Phase 5 Validation
- [ ] Webhook server starts on `WEBHOOK_PORT` (default 8080)
- [ ] Push events post commit summaries to #dev-announcements
- [ ] PR and issue events post formatted embeds
- [ ] Release events post to #announcements
- [ ] Signature verification rejects unsigned payloads
- [ ] `/repo`, `/issues`, `/commits` commands return GitHub data

---

## Phase 6: Engagement & Economy (Episode 8-9)

> Currency system, leaderboards, and mini-games.

**Branch**: `feat/economy`
**Depends on**: Phase 1 complete (database needed)

### 6.1 — Currency System

- [ ] **6.1.1** Create `src/commands/economy.rs`:
  - `/balance` — Check your Crimson Coins balance
  - `/leaderboard` — Top 10 earners embed
  - `/give @user [amount]` — Transfer coins to another member
  - `/daily` — Daily login bonus (with 24h cooldown)
  - `/coins add @user [amount]` (Admin) — Add coins
  - `/coins remove @user [amount]` (Admin) — Remove coins
  - `/coins reset @user` (Admin) — Reset balance

- [ ] **6.1.2** Earning methods (integrated into message handler):
  - 1 coin per message (5 min cooldown per user)
  - Voice channel presence tracking (optional)
  - Event participation bonuses

- [ ] **6.1.3** All transactions stored in `currency_transactions` table
- [ ] **6.1.4** Use `members` table for balance storage

### 6.2 — Leaderboards

- [ ] **6.2.1** Multiple leaderboard types: currency, message count, voice hours
- [ ] **6.2.2** Weekly auto-post option (scheduled Tokio task)

### 6.3 — Mini-Games

- [ ] **6.3.1** `/coinflip [amount]` — Double or nothing
- [ ] **6.3.2** `/guess` — Number guessing (1-100)
- [ ] **6.3.3** `/rps @user [amount]` — Rock-paper-scissors challenge (button interactions)
- [ ] **6.3.4** `/trivia` — Gaming/coding trivia with timed responses

### 6.4 — Leveling System (Optional)

- [ ] **6.4.1** XP from chat + voice activity
- [ ] **6.4.2** Level-up announcements in #general
- [ ] **6.4.3** Milestone roles at levels 5, 10, 25, 50

### Phase 6 Validation
- [ ] `/balance`, `/daily`, `/give` work correctly
- [ ] Coins earned passively from chat activity
- [ ] Leaderboard displays top members
- [ ] Mini-games cost/reward coins correctly
- [ ] Transaction history maintained in database

---

## Phase 7: Advanced Integrations (Episode 10+)

> Stretch goals with higher complexity.

**Branch**: `feat/advanced-integrations`
**Depends on**: Phases 1-2 complete

### 7.1 — Twitch Chat Bridge

- [ ] **7.1.1** Connect to Twitch IRC/TMI alongside EventSub
- [ ] **7.1.2** Bridge Discord `#live-chat` messages to Twitch (prefixed with `[Discord]`)
- [ ] **7.1.3** Bridge Twitch messages to Discord as webhook embeds
- [ ] **7.1.4** Only active when stream is live

### 7.2 — Clip Submission System

- [ ] **7.2.1** `/clip [twitch-clip-url]` — Submit clip to #clips
- [ ] **7.2.2** Voting via thumbs up/down reactions
- [ ] **7.2.3** Weekly "Clip of the Week" selection (scheduled task)

### 7.3 — StreamElements Integration

- [ ] **7.3.1** Connect to StreamElements WebSocket API
- [ ] **7.3.2** Post follow/sub/cheer/tip alerts to `#stream-activity`

### 7.4 — Custom Embeds Command

- [ ] **7.4.1** `/embed create` (Mod) — Interactive embed builder
- [ ] **7.4.2** `/embed edit [message_id]` (Mod) — Edit existing bot embed
- [ ] **7.4.3** `/embed send [channel]` (Mod) — Send built embed to channel

### Phase 7 Validation
- [ ] Chat bridge works bidirectionally during live streams
- [ ] Clips can be submitted and voted on
- [ ] StreamElements alerts post to Discord
- [ ] Custom embed builder creates valid embeds

---

## Cross-Cutting Concerns

### Security (All Phases)
- [ ] Never commit tokens/secrets to Git (.env in .gitignore)
- [ ] HMAC-SHA256 verification on all webhook payloads
- [ ] Rate-limit commands via Poise cooldowns
- [ ] Parameterized queries only (sqlx compile-time checking)
- [ ] Least-privilege Discord permissions and Twitch scopes
- [ ] Log all mod actions with moderator ID
- [ ] Sanitize embed content to prevent @everyone/@here injection
- [ ] Set `required_permissions` on all privileged slash commands

### Testing Strategy
- [ ] Unit tests for `config.rs`, `embeds.rs`, `permissions.rs`
- [ ] Integration tests for database migrations
- [ ] Manual testing against a staging Discord server
- [ ] `cargo clippy` + `cargo fmt` on every commit

### Deployment
- [ ] Update Dockerfile for new dependencies (sqlx, axum)
- [ ] Update Railway config for webhook port exposure (Phase 5)
- [ ] SQLite database file persistence between deploys
- [ ] Consider migration to self-hosted Linux when moving off Railway

---

## Dependency Graph

```
Phase 1.1 (Poise Migration) ──────┬──→ Phase 1.2 (Database) ──┬──→ Phase 1.4 (Welcome)
                                   │                            │
                                   ├──→ Phase 1.3 (Embeds) ────┤──→ Phase 1.5 (Info Commands)
                                   │                            │
                                   │                            ├──→ Phase 3 (Roles)
                                   │                            ├──→ Phase 4 (Moderation)
                                   │                            └──→ Phase 6 (Economy)
                                   │
                                   ├──→ Phase 2 (Twitch) ──────────→ Phase 7.1 (Chat Bridge)
                                   │                                  Phase 7.3 (StreamElements)
                                   │
                                   └──→ Phase 5 (GitHub)

Legend: ──→ = depends on
```

### Parallelization Opportunities
- **Phase 1.2 + 1.3** can be built in parallel (database and embeds are independent)
- **Phase 3 + Phase 4** can be built in parallel after Phase 1 completes
- **Phase 2 + Phase 3** can be built in parallel (Twitch is independent of Roles)
- **Phase 5** can be built any time after Phase 1 (independent of Twitch/Roles/Mod)
- **Phase 6** can be built any time after Phase 1 database is ready

---

## Execution Order (Recommended)

| Order | Task | Branch | Stream Episode |
|-------|------|--------|---------------|
| 1 | Phase 1.1 — Poise Migration + Restructure | `feat/poise-migration` | Ep 1 |
| 2 | Phase 1.2 — Database Layer | `feat/database-layer` | Ep 1 |
| 3 | Phase 1.3 — Embed Theme System | `feat/embed-system` | Ep 1 |
| 4 | Phase 1.4 — Welcome System | `feat/welcome-system` | Ep 2 |
| 5 | Phase 1.5 — Info Commands | `feat/info-commands` | Ep 2 |
| 6 | Phase 2 — Twitch Integration | `feat/twitch-integration` | Ep 3 |
| 7 | Phase 3 — Role Management | `feat/role-management` | Ep 4 |
| 8 | Phase 4.2 — Auto-Moderation | `feat/auto-mod` | Ep 5 |
| 9 | Phase 4.1 + 4.3 — Mod Commands + Logging | `feat/mod-commands` | Ep 6 |
| 10 | Phase 5 — GitHub Integration | `feat/github-integration` | Ep 7 |
| 11 | Phase 6.1 — Currency System | `feat/economy` | Ep 8 |
| 12 | Phase 6.3 — Mini-Games | `feat/mini-games` | Ep 9 |
| 13 | Phase 7.1 — Twitch Chat Bridge | `feat/chat-bridge` | Ep 10 |
| 14 | Phase 7.2-7.4 — Clips, StreamElements, Custom Embeds | `feat/advanced` | Ongoing |

---

## Environment Variables (Cumulative)

| Variable | Required | Phase | Description |
|----------|----------|-------|-------------|
| `DISCORD_TOKEN` | Yes | 1 | Discord bot token |
| `DATABASE_URL` | Yes | 1 | SQLite path (e.g. `sqlite:crimsonbot.db`) |
| `AUTOROLE_IDS` | No | 1 | Comma-separated role IDs for auto-assign |
| `TWITCH_CLIENT_ID` | Phase 2 | 2 | Twitch application client ID |
| `TWITCH_CLIENT_SECRET` | Phase 2 | 2 | Twitch application client secret |
| `TWITCH_CHANNEL_ID` | Phase 2 | 2 | Broadcaster user ID |
| `WEBHOOK_PORT` | Phase 5 | 5 | HTTP webhook server port (default: 8080) |
| `GITHUB_WEBHOOK_SECRET` | Phase 5 | 5 | Secret for verifying GitHub payloads |
| `GITHUB_TOKEN` | No | 5 | GitHub PAT for API calls (optional, increases rate limit) |
| `RUST_LOG` | No | — | Log level filter |

---

## Next Step

Run `/sc:implement` to begin executing Phase 1.1 (Poise Migration & Restructure).

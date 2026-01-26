# CrimsonX Discord Bot - Implementation Workflow

## Learning-Focused Development Plan

This workflow is designed for **learning Rust** while building a real project. Each phase introduces new Rust concepts progressively, with the **auto-role feature as your first working milestone**.

---

## Rust Concepts Roadmap

| Phase | New Rust Concepts |
|-------|-------------------|
| 0 | Cargo, modules, basic syntax, ownership basics |
| 1 | Structs, enums, traits, Result/Option, async/await |
| 2 | Lifetimes, generics, derive macros, error handling |
| 3 | Closures, iterators, pattern matching |
| 4 | Advanced async, channels, concurrent data structures |
| 5+ | Macros, unsafe (if needed), optimization |

---

## Phase 0: Environment & First Bot Connection
**Goal**: Get a bot online and responding to events
**Rust Concepts**: Cargo basics, modules, `fn main`, basic types

### 0.1 Development Environment Setup
- [x] Install Rust via rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [x] Install PostgreSQL (or use Docker: `docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=dev postgres:16`)
- [x] Install sqlx-cli: `cargo install sqlx-cli --no-default-features --features postgres`
- [x] Install cargo-watch for auto-reload: `cargo install cargo-watch`
- [x] Create Discord Application at https://discord.com/developers/applications
- [x] Enable required intents: Server Members, Message Content, Presence

**Learning Resources**:
- [The Rust Book - Getting Started](https://doc.rust-lang.org/book/ch01-00-getting-started.html)
- [Cargo Guide](https://doc.rust-lang.org/cargo/guide/)

### 0.2 Project Structure Setup
- [x] Update `Cargo.toml` with initial dependencies
- [x] Create directory structure:
  ```
  src/
  ├── main.rs           # Entry point
  ├── lib.rs            # Library root (re-exports)
  └── config.rs         # Configuration (start simple)
  ```
- [x] Create `.env` file for secrets
- [ ] Create `config/config.toml` with basic settings

**Checkpoint**: `cargo build` succeeds with no errors

### 0.3 Hello Discord - First Connection
- [ ] Write minimal `main.rs` that connects to Discord
- [ ] Implement basic event handler that logs "Ready!" when bot starts
- [ ] Test bot comes online in your server

```rust
// What you'll learn:
// - async fn main with #[tokio::main]
// - Creating structs (Client)
// - Method calls and .await
// - Basic error handling with .expect()
```

**Checkpoint**: Bot shows as online in Discord

### 0.4 First Event Handler - Member Join Detection
- [ ] Add GuildMemberAdd event handler
- [ ] Log when someone joins (just println! for now)
- [ ] Test by having someone join (or use a test server)

```rust
// What you'll learn:
// - Implementing traits (EventHandler)
// - async trait methods
// - Reading data from event structs
// - &self references
```

**Checkpoint**: Console shows message when member joins

---

## Phase 1: Auto-Role Feature (First Complete Feature!)
**Goal**: Automatically assign a role when users join
**Rust Concepts**: Structs, enums, Option, Result, basic async, environment variables

### 1.1 Configuration Loading
- [ ] Create `config.rs` with Config struct
- [ ] Load from environment variables (simpler than TOML for now)
- [ ] Add `AUTOROLE_ID` environment variable
- [ ] Parse role ID from string to u64

```rust
// What you'll learn:
// - Defining structs with #[derive(Debug, Clone)]
// - Option<T> for optional values
// - std::env::var() and handling errors
// - String parsing with .parse::<u64>()
```

### 1.2 Implement Role Assignment
- [ ] On member join, fetch the role from config
- [ ] Use Serenity's `member.add_role()` method
- [ ] Handle the case where role doesn't exist

```rust
// What you'll learn:
// - Working with Result<T, E>
// - The ? operator for error propagation
// - match expressions
// - Serenity's GuildId, RoleId types (newtypes)
```

### 1.3 Error Handling
- [ ] Create basic `error.rs` with custom Error enum
- [ ] Handle permission errors gracefully
- [ ] Log errors instead of crashing

```rust
// What you'll learn:
// - Defining enums with variants
// - #[derive(Debug, thiserror::Error)]
// - impl From<T> for error conversion
// - The anyhow crate for application errors
```

### 1.4 Add Logging
- [ ] Set up tracing subscriber
- [ ] Replace println! with tracing macros (info!, warn!, error!)
- [ ] Add structured logging fields

```rust
// What you'll learn:
// - Crate initialization patterns
// - Macros (info!, debug!, etc.)
// - Structured logging with fields
```

**MILESTONE CHECKPOINT**:
Bot automatically assigns a role when members join. This is your first complete feature!

**Test Plan**:
1. Set `AUTOROLE_ID` to a valid role in your server
2. Have someone join (or use a test account)
3. Verify they receive the role
4. Check logs for confirmation message

---

## Phase 2: Database Integration
**Goal**: Store auto-roles in database (multiple roles, configurable)
**Rust Concepts**: Lifetimes, generics, derive macros, SQL with sqlx

### 2.1 Database Connection
- [ ] Set up sqlx with PostgreSQL
- [ ] Create connection pool
- [ ] Test connection on startup

```rust
// What you'll learn:
// - Connection pools and why they matter
// - sqlx::PgPool type
// - async connection handling
```

### 2.2 First Migration
- [ ] Create `migrations/` directory
- [ ] Write migration for `auto_roles` table
- [ ] Run migration with sqlx-cli

```sql
-- migrations/001_auto_roles.sql
CREATE TABLE auto_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    trigger TEXT NOT NULL DEFAULT 'join',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(guild_id, role_id, trigger)
);
```

### 2.3 Database Models
- [ ] Create `models/auto_role.rs`
- [ ] Define AutoRole struct with sqlx derives
- [ ] Implement CRUD operations

```rust
// What you'll learn:
// - #[derive(sqlx::FromRow)]
// - Lifetime annotations in queries
// - Generic return types
// - SQL query macros
```

### 2.4 Integrate with Event Handler
- [ ] Query database on member join
- [ ] Assign all roles with 'join' trigger
- [ ] Handle multiple roles

```rust
// What you'll learn:
// - Sharing database pool across handlers (Arc)
// - Iterating over query results
// - for loop with async operations
```

**Checkpoint**: Auto-roles are now database-driven

---

## Phase 3: Slash Commands with Poise
**Goal**: Add `/autorole add` and `/autorole list` commands
**Rust Concepts**: Closures, iterators, pattern matching, Poise framework

### 3.1 Poise Framework Setup
- [ ] Add Poise to the project
- [ ] Create command framework structure
- [ ] Set up application command registration

```rust
// What you'll learn:
// - Generic type parameters (Data, Error)
// - Framework builders
// - Application command registration
```

### 3.2 Create Commands Module
- [ ] Set up `commands/mod.rs`
- [ ] Create `commands/autorole.rs`
- [ ] Implement `/autorole add <role>` command

```rust
// What you'll learn:
// - Poise command macros
// - Command parameters with validation
// - Context types (ApplicationContext)
// - Responding to interactions
```

### 3.3 Implement /autorole list
- [ ] Query all auto-roles for guild
- [ ] Format as Discord embed
- [ ] Handle empty list case

```rust
// What you'll learn:
// - Iterators (.iter(), .map(), .collect())
// - Closures |x| { ... }
// - Building embeds with builder pattern
```

### 3.4 Implement /autorole remove
- [ ] Delete auto-role from database
- [ ] Confirm deletion to user
- [ ] Handle "not found" case

```rust
// What you'll learn:
// - DELETE queries with sqlx
// - Handling affected row counts
// - Pattern matching on Option
```

### 3.5 Permission Checks
- [ ] Add permission requirement (Manage Roles)
- [ ] Create reusable permission check function

```rust
// What you'll learn:
// - Poise check functions
// - Bitflags (Discord permissions)
// - Early returns
```

**Checkpoint**: Complete auto-role system with commands

---

## Phase 4: Welcome Messages
**Goal**: Send welcome messages when users join
**Rust Concepts**: String formatting, template parsing, more pattern matching

### 4.1 Guild Config Table
- [ ] Create migration for `guild_configs`
- [ ] Add welcome_channel_id, welcome_message columns
- [ ] Create GuildConfig model

### 4.2 Template Variables
- [ ] Implement simple template parser
- [ ] Support `{user}`, `{server}`, `{member_count}`
- [ ] Handle missing variables gracefully

```rust
// What you'll learn:
// - String manipulation (.replace(), format!())
// - Regex (optional, can use simple replace)
// - Handling user input safely
```

### 4.3 Welcome Commands
- [ ] `/welcome channel <channel>` - Set channel
- [ ] `/welcome message <text>` - Set message
- [ ] `/welcome test` - Preview message

### 4.4 Welcome Event
- [ ] Send welcome message on member join
- [ ] Apply template variables
- [ ] Handle channel not found

**Checkpoint**: Welcome messages working

---

## Phase 5: Moderation Basics
**Goal**: Implement warn, kick, ban with logging
**Rust Concepts**: More complex data structures, time handling, embeds

### 5.1 Mod Actions Table
- [ ] Create migration for `mod_actions`
- [ ] Create ModAction model with action types enum

### 5.2 Basic Commands
- [ ] `/warn <user> <reason>`
- [ ] `/kick <user> [reason]`
- [ ] `/ban <user> [reason]`

### 5.3 Mod Log
- [ ] Create formatted embed for mod actions
- [ ] Post to configured mod log channel
- [ ] Include case number, moderator, reason

### 5.4 User History
- [ ] `/history <user>` - View past actions
- [ ] Paginated embed for many actions

**Checkpoint**: Basic moderation working

---

## Phase 6: Leveling System
**Goal**: XP gain from messages, reactions, voice
**Rust Concepts**: Concurrent data structures, background tasks, caching

### 6.1 XP Tracking
- [ ] Create members and member_levels tables
- [ ] Track XP with cooldown
- [ ] Calculate level from XP

### 6.2 Message XP Handler
- [ ] Award XP on message (with cooldown)
- [ ] Use DashMap for cooldown cache
- [ ] Announce level-ups

### 6.3 Voice XP (Background Task)
- [ ] Track voice channel time
- [ ] Background task to award XP
- [ ] Handle disconnects

### 6.4 Rank Command
- [ ] `/rank [user]` with embed
- [ ] Progress bar with Unicode
- [ ] Leaderboard position

### 6.5 Leaderboard
- [ ] `/leaderboard [page]`
- [ ] Top 10 per page
- [ ] Button navigation

**Checkpoint**: Full leveling system

---

## Phase 7: Role Menus
**Goal**: Interactive role selection with buttons/dropdowns
**Rust Concepts**: Component interactions, serialization, complex state

### 7.1 Role Menu Creation
- [ ] `/rolemenu create` command
- [ ] Modal for title/description
- [ ] Add roles with buttons

### 7.2 Component Handlers
- [ ] Handle button clicks
- [ ] Handle select menu selections
- [ ] Toggle roles on/off

### 7.3 Persistence
- [ ] Store role menus in database
- [ ] Restore on bot restart
- [ ] Handle deleted messages/roles

**Checkpoint**: Interactive role menus working

---

## Phase 8: Custom Commands
**Goal**: User-created commands with text/embed responses
**Rust Concepts**: Dynamic dispatch, command routing

### 8.1 Custom Command Storage
- [ ] Database table for commands
- [ ] Create/delete operations

### 8.2 Command Execution
- [ ] Intercept non-slash messages (or create dynamic slash commands)
- [ ] Execute matching custom commands
- [ ] Track usage count

**Checkpoint**: Custom commands working

---

## Phase 9: Twitch Integration
**Goal**: Stream notifications and live role
**Rust Concepts**: HTTP clients, OAuth, background polling

### 9.1 Twitch API Client
- [ ] OAuth authentication
- [ ] Stream status endpoint
- [ ] Token refresh

### 9.2 Stream Poller
- [ ] Background task to poll every 60s
- [ ] Detect live/offline transitions
- [ ] Post notifications

### 9.3 Live Role
- [ ] Assign role when live
- [ ] Remove when offline

**Checkpoint**: Twitch integration complete

---

## Phase 10: Polish & Production
**Goal**: Production-ready deployment
**Rust Concepts**: Release builds, optimization, monitoring

### 10.1 Error Handling Review
- [ ] Audit all error paths
- [ ] Improve user messages
- [ ] Add error recovery

### 10.2 Performance
- [ ] Connection pool tuning
- [ ] Cache optimization
- [ ] Query analysis

### 10.3 Docker Deployment
- [ ] Multi-stage Dockerfile
- [ ] Docker Compose for dev
- [ ] Health checks

### 10.4 Monitoring
- [ ] Structured logging
- [ ] Metrics (optional)
- [ ] Alerting (optional)

---

## Quick Reference: Running the Bot

```bash
# Development (with auto-reload)
cargo watch -x run

# Check for errors without running
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Build release
cargo build --release

# Run database migrations
sqlx migrate run
```

---

## Validation Gates

Before moving to the next phase, ensure:

1. **Code compiles**: `cargo check` passes
2. **No warnings**: `cargo clippy` is clean (or warnings understood)
3. **Formatted**: `cargo fmt` applied
4. **Feature works**: Manual testing in Discord
5. **Concepts understood**: Can explain the Rust concepts used

---

## Resources for Learning

### Official
- [The Rust Book](https://doc.rust-lang.org/book/) - Read chapters as needed
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Serenity Documentation](https://docs.rs/serenity)
- [Poise Guide](https://github.com/serenity-rs/poise/wiki)

### When Stuck
- [Rust Users Forum](https://users.rust-lang.org/)
- [Serenity Discord](https://discord.gg/serenity-rs)
- Error messages in Rust are usually very helpful - read them carefully!

### IDE Setup
- VS Code with rust-analyzer extension (recommended)
- Enable "check on save" for immediate feedback

---

## Next Step

Ready to start? Run `/sc:implement` and ask to begin with **Phase 0.1** (Environment Setup).

Take your time with each phase - the goal is learning Rust, not just finishing the bot!

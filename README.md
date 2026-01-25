# CrimsonX

A monorepo for projects developed on the [CrimsonX Twitch channel](https://www.twitch.tv/0xdc143c) — building developer tools and community software live on stream.

## Projects

### Discord Bot

A high-performance Discord bot built in Rust for managing Twitch community servers. Designed to scale from small communities to large, active servers.

**Status:** In Development

**Features:**
- **Leveling System** — XP from messages, reactions, and voice chat with customizable rewards and leaderboards
- **Moderation** — Warn, mute, kick, ban with comprehensive mod logs and case tracking
- **Role Menus** — Interactive button and dropdown-based role selection
- **Auto-Roles** — Automatic role assignment on join, boost, or verification
- **Welcome/Leave Messages** — Customizable greetings with template variables
- **Custom Commands** — Guild-specific text and embed commands
- **Twitch Integration** — Stream notifications and live role assignment

**Tech Stack:**
| Component | Technology |
|-----------|------------|
| Language | Rust |
| Discord Library | Serenity + Poise |
| Database | PostgreSQL (sqlx) |
| Async Runtime | Tokio |
| Configuration | TOML |

See [`apps/discord-bot/docs/DESIGN.md`](apps/discord-bot/docs/DESIGN.md) for the full system design specification.

## Repository Structure

```
crimsonx/
├── apps/
│   └── discord-bot/     # Rust Discord bot
├── nx.json              # Nx workspace configuration
└── mise.toml            # Tool version management
```

This is an [Nx](https://nx.dev) monorepo, allowing multiple projects to coexist with shared tooling and efficient task orchestration.

## Development

### Prerequisites

- [Rust](https://rustup.rs/) 1.93.0+
- [mise](https://mise.jdx.dev/) (recommended for tool version management)
- [Nx](https://nx.dev) CLI

### Getting Started

```bash
# Install Rust toolchain (handled by mise if configured)
mise install

# Run tasks via Nx
nx run discord-bot:build
nx run discord-bot:test
```

### Version Control

This project uses [Jujutsu (jj)](https://github.com/martinvonz/jj) for version control:

```bash
jj status          # Check status
jj diff            # View changes
jj new             # Create new change
jj describe -m ""  # Add description
jj git push        # Push to remote
```

## Connect

- **Twitch:** [twitch.tv/0xdc143c](https://www.twitch.tv/0xdc143c)
- **Discord:** Coming soon

## License

This project is open source. License details coming soon.

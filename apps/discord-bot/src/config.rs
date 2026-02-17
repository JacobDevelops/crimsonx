use crate::error::Error;
use serenity::all::{ChannelId, GuildId, RoleId};

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub database_url: String,
    pub autorole_ids: Vec<RoleId>,
    pub guild_id: Option<GuildId>,
    pub welcome_channel_id: Option<ChannelId>,
    pub log_channel_id: Option<ChannelId>,
    pub bot_version: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required:
    /// - `DISCORD_TOKEN` — Bot token from Discord Developer Portal
    /// - `DATABASE_URL` — SQLite connection string (e.g. "sqlite:crimsonbot.db")
    ///
    /// Optional:
    /// - `AUTOROLE_IDS` — Comma-separated role IDs to assign on join
    /// - `WELCOME_CHANNEL_ID` — Channel for welcome embeds
    /// - `LOG_CHANNEL_ID` — Channel for mod-logs
    pub fn from_env() -> Result<Self, Error> {
        let discord_token = std::env::var("DISCORD_TOKEN")
            .map_err(|_| Error::Config("DISCORD_TOKEN environment variable is required".into()))?;

        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:crimsonbot.db".into());

        let autorole_ids = parse_id_list::<RoleId>("AUTOROLE_IDS")?;

        let guild_id = parse_optional_id::<GuildId>("GUILD_ID")?;
        let welcome_channel_id = parse_optional_id::<ChannelId>("WELCOME_CHANNEL_ID")?;
        let log_channel_id = parse_optional_id::<ChannelId>("LOG_CHANNEL_ID")?;

        Ok(Self {
            discord_token,
            database_url,
            autorole_ids,
            guild_id,
            welcome_channel_id,
            log_channel_id,
            bot_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

fn parse_id_list<T>(var: &str) -> Result<Vec<T>, Error>
where
    T: From<u64>,
{
    match std::env::var(var) {
        Ok(val) if !val.is_empty() => val
            .split(',')
            .map(|s| {
                s.trim()
                    .parse::<u64>()
                    .map(T::from)
                    .map_err(|_| Error::Config(format!("Invalid ID in {var}: '{s}'")))
            })
            .collect(),
        _ => Ok(Vec::new()),
    }
}

fn parse_optional_id<T>(var: &str) -> Result<Option<T>, Error>
where
    T: From<u64>,
{
    match std::env::var(var) {
        Ok(val) if !val.is_empty() => {
            let id = val
                .trim()
                .parse::<u64>()
                .map_err(|_| Error::Config(format!("Invalid ID for {var}: '{val}'")))?;
            Ok(Some(T::from(id)))
        }
        _ => Ok(None),
    }
}

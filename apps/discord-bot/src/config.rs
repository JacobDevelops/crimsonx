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
    // Twitch (Phase 2)
    pub twitch: Option<TwitchConfig>,
}

#[derive(Debug, Clone)]
pub struct TwitchConfig {
    pub client_id: String,
    pub client_secret: String,
    pub channel_id: String,
    pub live_channel_id: ChannelId,
    pub live_chat_channel_id: Option<ChannelId>,
    pub live_role_id: Option<RoleId>,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required:
    /// - `DISCORD_TOKEN` — Bot token from Discord Developer Portal
    ///
    /// Optional:
    /// - `DATABASE_URL` — PostgreSQL connection string
    /// - `AUTOROLE_IDS` — Comma-separated role IDs to assign on join
    /// - `GUILD_ID` — Guild for instant slash command registration
    /// - `WELCOME_CHANNEL_ID` — Channel for welcome embeds
    /// - `LOG_CHANNEL_ID` — Channel for mod-logs
    /// - `TWITCH_CLIENT_ID` + `TWITCH_CLIENT_SECRET` + `TWITCH_CHANNEL_ID` + `LIVE_CHANNEL_ID` — Twitch integration
    pub fn from_env() -> Result<Self, Error> {
        let discord_token = std::env::var("DISCORD_TOKEN")
            .map_err(|_| Error::Config("DISCORD_TOKEN environment variable is required".into()))?;

        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:crimsonbot.db".into());

        let autorole_ids = parse_id_list::<RoleId>("AUTOROLE_IDS")?;

        let guild_id = parse_optional_id::<GuildId>("GUILD_ID")?;
        let welcome_channel_id = parse_optional_id::<ChannelId>("WELCOME_CHANNEL_ID")?;
        let log_channel_id = parse_optional_id::<ChannelId>("LOG_CHANNEL_ID")?;

        let twitch = TwitchConfig::from_env()?;

        Ok(Self {
            discord_token,
            database_url,
            autorole_ids,
            guild_id,
            welcome_channel_id,
            log_channel_id,
            bot_version: env!("CARGO_PKG_VERSION").to_string(),
            twitch,
        })
    }
}

impl TwitchConfig {
    fn from_env() -> Result<Option<Self>, Error> {
        let client_id = match std::env::var("TWITCH_CLIENT_ID") {
            Ok(v) if !v.is_empty() => v,
            _ => return Ok(None),
        };
        let client_secret = std::env::var("TWITCH_CLIENT_SECRET").map_err(|_| {
            Error::Config(
                "TWITCH_CLIENT_SECRET is required when TWITCH_CLIENT_ID is set".into(),
            )
        })?;
        let channel_id = std::env::var("TWITCH_CHANNEL_ID").map_err(|_| {
            Error::Config("TWITCH_CHANNEL_ID is required when TWITCH_CLIENT_ID is set".into())
        })?;
        let live_channel_id = parse_optional_id::<ChannelId>("LIVE_CHANNEL_ID")?.ok_or_else(
            || Error::Config("LIVE_CHANNEL_ID is required when TWITCH_CLIENT_ID is set".into()),
        )?;
        let live_chat_channel_id = parse_optional_id::<ChannelId>("LIVE_CHAT_CHANNEL_ID")?;
        let live_role_id = parse_optional_id::<RoleId>("LIVE_ROLE_ID")?;

        Ok(Some(Self {
            client_id,
            client_secret,
            channel_id,
            live_channel_id,
            live_chat_channel_id,
            live_role_id,
        }))
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

use serenity::all::CreateEmbed;

/// CrimsonX brand colors used across all bot embeds.
pub struct Colors;

impl Colors {
    pub const CRIMSON: u32 = 0xDC143C;
    pub const SUCCESS: u32 = 0x00FF7F;
    pub const WARNING: u32 = 0xFFD700;
    pub const ERROR: u32 = 0xFF4444;
    pub const MODERATION: u32 = 0x8B0A1E;
    pub const TWITCH: u32 = 0x9146FF;
    pub const GITHUB: u32 = 0x238636;
    pub const ECONOMY: u32 = 0x00CED1;
}

/// Create a standard CrimsonX-themed embed with default color, footer, and timestamp.
pub fn crimson_embed() -> CreateEmbed {
    base_embed(Colors::CRIMSON)
}

/// Create a success-themed embed (green).
pub fn success_embed() -> CreateEmbed {
    base_embed(Colors::SUCCESS)
}

/// Create a warning-themed embed (gold).
pub fn warning_embed() -> CreateEmbed {
    base_embed(Colors::WARNING)
}

/// Create an error-themed embed (red).
pub fn error_embed() -> CreateEmbed {
    base_embed(Colors::ERROR)
}

/// Create a moderation-themed embed (dark crimson).
pub fn moderation_embed() -> CreateEmbed {
    base_embed(Colors::MODERATION)
}

/// Create a Twitch-themed embed (purple).
pub fn twitch_embed() -> CreateEmbed {
    base_embed(Colors::TWITCH)
}

/// Create a GitHub-themed embed (green).
pub fn github_embed() -> CreateEmbed {
    base_embed(Colors::GITHUB)
}

/// Create an economy-themed embed (teal).
pub fn economy_embed() -> CreateEmbed {
    base_embed(Colors::ECONOMY)
}

fn base_embed(color: u32) -> CreateEmbed {
    CreateEmbed::default()
        .color(color)
        .footer(serenity::all::CreateEmbedFooter::new(
            "CrimsonX \u{2022} 0xDC143C",
        ))
        .timestamp(serenity::model::Timestamp::now())
}

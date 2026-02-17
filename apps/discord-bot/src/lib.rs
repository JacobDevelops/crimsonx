pub mod commands;
pub mod config;
pub mod db;
pub mod error;
pub mod events;
pub mod utils;

use sqlx::PgPool;

/// Shared data accessible across all Poise commands and event handlers.
pub struct Data {
    pub db: PgPool,
    pub config: config::Config,
    pub start_time: std::time::Instant,
}

/// Poise context alias used throughout the bot.
pub type Context<'a> = poise::Context<'a, Data, error::Error>;

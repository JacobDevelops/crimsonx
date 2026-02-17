use discord_bot::commands;
use discord_bot::config::Config;
use discord_bot::events;
use discord_bot::Data;
use poise::serenity_prelude as serenity;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "discord_bot=info".parse().unwrap()),
        )
        .init();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            error!(error = %e, "Failed to load configuration");
            std::process::exit(1);
        }
    };

    if config.autorole_ids.is_empty() {
        warn!("No AUTOROLE_IDS configured — auto-role assignment is disabled");
    } else {
        info!(roles = ?config.autorole_ids, "Auto-role assignment enabled");
    }

    let db = match discord_bot::db::init_pool(&config.database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            error!(error = %e, "Failed to initialize database");
            std::process::exit(1);
        }
    };

    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS
        | serenity::GatewayIntents::GUILD_VOICE_STATES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::general::ping(),
                commands::general::about(),
                commands::general::socials(),
                commands::general::schedule(),
                commands::general::server(),
                commands::general::help(),
            ],
            event_handler: |ctx, event, _framework, data| {
                Box::pin(async move {
                    events::member::handle_event(ctx, event, data).await;
                    Ok(())
                })
            },
            on_error: |error| {
                Box::pin(async move {
                    match error {
                        poise::FrameworkError::Command { error, ctx, .. } => {
                            let embed = discord_bot::utils::embeds::error_embed()
                                .title("Error")
                                .description(error.user_message());
                            let _ = ctx
                                .send(poise::CreateReply::default().embed(embed).ephemeral(true))
                                .await;
                            tracing::error!(
                                command = ctx.command().name,
                                error = %error,
                                "Command error"
                            );
                        }
                        other => {
                            if let Err(e) = poise::builtins::on_error(other).await {
                                tracing::error!(error = %e, "Error handling error");
                            }
                        }
                    }
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                info!(bot = %ready.user.name, guilds = ready.guilds.len(), "Bot is ready!");

                // Register slash commands (guild-specific if GUILD_ID set, otherwise global)
                if let Some(guild_id) = config.guild_id {
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await?;
                    info!(guild_id = %guild_id, "Slash commands registered to guild");
                } else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    info!("Slash commands registered globally");
                }

                // Set bot status
                ctx.set_activity(Some(serenity::ActivityData::watching("the crimson tide")));

                Ok(Data {
                    db,
                    config,
                    start_time: std::time::Instant::now(),
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(config_token(), intents)
        .framework(framework)
        .await
        .expect("Failed to create Discord client");

    // Graceful shutdown on SIGINT/SIGTERM
    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        info!("Shutdown signal received, stopping bot...");
        shard_manager.shutdown_all().await;
    });

    info!("Starting bot...");
    if let Err(why) = client.start().await {
        error!(error = %why, "Client error");
    }
    info!("Bot has shut down cleanly");
}

/// Load the Discord token for client builder (needed because config is moved into setup).
fn config_token() -> String {
    std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set")
}

/// Wait for a shutdown signal (SIGINT or SIGTERM).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

use crate::utils::embeds;
use crate::Context;

type Error = crate::error::Error;

/// Check bot latency.
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let msg = ctx.say("Pong!").await?;
    let api_latency = start.elapsed().as_millis();

    let embed = embeds::crimson_embed().title("Pong!").field(
        "API Latency",
        format!("{}ms", api_latency),
        true,
    );

    msg.edit(ctx, poise::CreateReply::default().content("").embed(embed))
        .await?;

    Ok(())
}

/// Show bot info and credits.
#[poise::command(slash_command, prefix_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let uptime = ctx.data().start_time.elapsed();
    let hours = uptime.as_secs() / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;
    let seconds = uptime.as_secs() % 60;

    let embed = embeds::crimson_embed()
        .title("About CrimsonBot")
        .description("Custom Discord bot for The Crimson Den, built in Rust.")
        .field("Version", &ctx.data().config.bot_version, true)
        .field("Uptime", format!("{hours}h {minutes}m {seconds}s"), true)
        .field("Language", "Rust + Serenity/Poise", true)
        .field(
            "Source",
            "[GitHub Repository](https://github.com/0xDC143C/crimsonx)",
            false,
        );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Links to all CrimsonX social platforms.
#[poise::command(slash_command, prefix_command)]
pub async fn socials(ctx: Context<'_>) -> Result<(), Error> {
    let embed = embeds::crimson_embed()
        .title("CrimsonX Socials")
        .description("Follow CrimsonX across all platforms!")
        .field(
            "Twitch",
            "[twitch.tv/0xDC143C](https://twitch.tv/0xDC143C)",
            true,
        )
        .field("YouTube", "[YouTube](https://youtube.com/@0xDC143C)", true)
        .field("X (Twitter)", "[@0xDC143C](https://x.com/0xDC143C)", true)
        .field(
            "GitHub",
            "[github.com/0xDC143C](https://github.com/0xDC143C)",
            true,
        );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Show the current streaming schedule.
#[poise::command(slash_command, prefix_command)]
pub async fn schedule(ctx: Context<'_>) -> Result<(), Error> {
    let embed = embeds::twitch_embed()
        .title("Streaming Schedule")
        .description("Check back soon for the updated schedule!")
        .field("Status", "Schedule coming soon", false);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Show server info and stats.
#[poise::command(slash_command, prefix_command)]
pub async fn server(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().map(|g| {
        (
            g.name.clone(),
            g.member_count,
            g.premium_tier,
            g.premium_subscription_count,
            g.id.created_at(),
        )
    });

    let embed = if let Some((name, member_count, boost_tier, boost_count, created_at)) = guild {
        embeds::crimson_embed()
            .title(format!("{name} Server Info"))
            .field("Members", member_count.to_string(), true)
            .field("Boost Tier", format!("{boost_tier:?}"), true)
            .field("Boosts", boost_count.unwrap_or(0).to_string(), true)
            .field(
                "Created",
                format!("<t:{}:R>", created_at.unix_timestamp()),
                true,
            )
    } else {
        embeds::error_embed()
            .title("Error")
            .description("Could not fetch server information. Use this command in a server.")
    };

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// List all available commands.
#[poise::command(slash_command, prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Command to get help for"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "CrimsonBot — built with Rust + Poise for The Crimson Den",
            show_context_menu_commands: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

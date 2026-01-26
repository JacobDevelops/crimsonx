use crate::utils::embeds;
use crate::Data;
use rand::seq::SliceRandom;
use serenity::all::{Context, CreateMessage, FullEvent, Member, Mentionable};
use tracing::{error, info};

const WELCOME_MESSAGES: &[&str] = &[
    "Welcome to The Crimson Den, **{user}**!",
    "**{user}** just joined the crew!",
    "Glad to have you here, **{user}**!",
    "**{user}** has entered The Crimson Den!",
    "Welcome aboard, **{user}**!",
];

/// Handle member-related Discord events (join/leave).
pub async fn handle_event(ctx: &Context, event: &FullEvent, data: &Data) {
    match event {
        FullEvent::GuildMemberAddition { new_member } => {
            handle_member_join(ctx, new_member, data).await;
        }
        FullEvent::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available,
        } => {
            handle_member_leave(
                ctx,
                *guild_id,
                user,
                member_data_if_available.as_ref(),
                data,
            )
            .await;
        }
        _ => {}
    }
}

async fn handle_member_join(ctx: &Context, member: &Member, data: &Data) {
    let user_name = &member.user.name;
    let display_name = member.display_name();

    // 1. Auto-role assignment
    if !data.config.autorole_ids.is_empty() {
        for role_id in &data.config.autorole_ids {
            if let Err(why) = member.add_role(&ctx.http, *role_id).await {
                error!(
                    user = %user_name,
                    role_id = %role_id,
                    error = %why,
                    "Failed to assign auto-role"
                );
            }
        }
        info!(
            user = %user_name,
            roles = ?data.config.autorole_ids,
            "Assigned auto-roles to new member"
        );
    }

    // 2. Welcome embed in #welcome
    if let Some(channel_id) = data.config.welcome_channel_id {
        let welcome_msg = WELCOME_MESSAGES
            .choose(&mut rand::thread_rng())
            .unwrap_or(&WELCOME_MESSAGES[0])
            .replace("{user}", &member.mention().to_string());

        let member_count = member
            .guild_id
            .to_partial_guild(&ctx.http)
            .await
            .map(|g| g.approximate_member_count.unwrap_or(0))
            .unwrap_or(0);

        let embed = embeds::crimson_embed()
            .title("Welcome to The Crimson Den!")
            .description(welcome_msg)
            .thumbnail(
                member
                    .user
                    .avatar_url()
                    .unwrap_or_else(|| member.user.default_avatar_url()),
            )
            .field("Member #", member_count.to_string(), true)
            .field(
                "Quick Links",
                "Check out <#rules> and grab your roles in <#roles>!",
                false,
            );

        let message = CreateMessage::new().embed(embed);
        if let Err(why) = channel_id.send_message(&ctx.http, message).await {
            error!(error = %why, "Failed to send welcome message");
        }
    }

    // 3. Log join to #mod-logs
    if let Some(log_channel) = data.config.log_channel_id {
        let account_age = member.user.created_at();

        let embed = embeds::crimson_embed()
            .title("Member Joined")
            .field(
                "User",
                format!("{} ({})", member.mention(), user_name),
                false,
            )
            .field(
                "Account Created",
                format!("<t:{}:R>", account_age.unix_timestamp()),
                true,
            )
            .thumbnail(
                member
                    .user
                    .avatar_url()
                    .unwrap_or_else(|| member.user.default_avatar_url()),
            );

        let message = CreateMessage::new().embed(embed);
        if let Err(why) = log_channel.send_message(&ctx.http, message).await {
            error!(error = %why, "Failed to send join log");
        }
    }

    info!(user = %user_name, display_name = %display_name, "Member joined");
}

async fn handle_member_leave(
    ctx: &Context,
    guild_id: serenity::all::GuildId,
    user: &serenity::all::User,
    member: Option<&Member>,
    data: &Data,
) {
    let user_name = &user.name;

    if let Some(log_channel) = data.config.log_channel_id {
        let mut embed = embeds::warning_embed().title("Member Left").field(
            "User",
            format!("{} ({})", user.mention(), user_name),
            false,
        );

        if let Some(member) = member {
            let roles: Vec<String> = member.roles.iter().map(|r| format!("<@&{}>", r)).collect();

            if !roles.is_empty() {
                embed = embed.field("Roles", roles.join(", "), false);
            }

            if let Some(joined_at) = member.joined_at {
                embed = embed.field(
                    "Joined",
                    format!("<t:{}:R>", joined_at.unix_timestamp()),
                    true,
                );
            }
        }

        embed = embed.thumbnail(
            user.avatar_url()
                .unwrap_or_else(|| user.default_avatar_url()),
        );

        let message = CreateMessage::new().embed(embed);
        if let Err(why) = log_channel.send_message(&ctx.http, message).await {
            error!(error = %why, "Failed to send leave log");
        }
    }

    info!(user = %user_name, guild_id = %guild_id, "Member left");
}

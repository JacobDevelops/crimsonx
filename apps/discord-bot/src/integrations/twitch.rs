use crate::config::TwitchConfig;
use crate::utils::embeds;
use futures_util::{SinkExt, StreamExt};
use serenity::all::{
    ChannelId, Context as SerenityContext, CreateMessage, EditChannel, EditMessage, Mentionable,
    MessageId, PermissionOverwrite, PermissionOverwriteType, Permissions, RoleId,
};
use serenity::model::id::GuildId;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{error, info, warn};
use twitch_api::eventsub::{
    channel::ChannelUpdateV2, stream::StreamOfflineV1, stream::StreamOnlineV1, Event,
    EventsubWebsocketData, Message, Payload, Transport,
};
use twitch_api::helix::streams::GetStreamsRequest;
use twitch_api::twitch_oauth2::{AppAccessToken, ClientId, ClientSecret, TwitchToken};
use twitch_api::types::UserIdRef;
use twitch_api::HelixClient;

const TWITCH_EVENTSUB_URL: &str = "wss://eventsub.wss.twitch.tv/ws";

// ─── Twitch Client State ────────────────────────────────────────────

struct TwitchState {
    helix: HelixClient<'static, reqwest::Client>,
    token: Arc<RwLock<AppAccessToken>>,
    config: TwitchConfig,
}

impl TwitchState {
    async fn new(config: &TwitchConfig) -> Result<Self, String> {
        let helix: HelixClient<'static, reqwest::Client> = HelixClient::new();

        let token: AppAccessToken = AppAccessToken::get_app_access_token(
            &helix,
            ClientId::new(config.client_id.clone()),
            ClientSecret::new(config.client_secret.clone()),
            vec![],
        )
        .await
        .map_err(|e| format!("Failed to get Twitch app access token: {e}"))?;

        info!(
            expires_in = ?token.expires_in(),
            "Twitch app access token acquired"
        );

        Ok(Self {
            helix,
            token: Arc::new(RwLock::new(token)),
            config: config.clone(),
        })
    }

    fn spawn_refresh_loop(&self) {
        let helix = self.helix.clone();
        let token: Arc<RwLock<AppAccessToken>> = Arc::clone(&self.token);

        tokio::spawn(async move {
            loop {
                let sleep_secs = {
                    let t: tokio::sync::RwLockReadGuard<'_, AppAccessToken> = token.read().await;
                    let expires = t.expires_in().as_secs();
                    // Refresh 5 minutes before expiry, minimum 60s
                    expires.saturating_sub(300).max(60)
                };

                tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)).await;

                let mut t: tokio::sync::RwLockWriteGuard<'_, AppAccessToken> = token.write().await;
                match t.refresh_token(&helix).await {
                    Ok(()) => {
                        info!(expires_in = ?t.expires_in(), "Twitch access token refreshed");
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to refresh Twitch token, retrying in 60s");
                    }
                }
            }
        });
    }

    async fn fetch_stream_info(
        &self,
    ) -> Result<Option<twitch_api::helix::streams::Stream>, String> {
        let token = self.token.read().await;
        let ids: &[&UserIdRef] = &[self.config.channel_id.as_str().into()];
        let req = GetStreamsRequest::user_ids(ids);

        let response = self
            .helix
            .req_get(req, &*token)
            .await
            .map_err(|e| format!("Helix GetStreams failed: {e}"))?;

        Ok(response.data.into_iter().next())
    }

    async fn subscribe_events(&self, session_id: &str) -> Result<(), String> {
        let token = self.token.read().await;
        let transport = Transport::websocket(session_id);
        let channel_id = self.config.channel_id.as_str();

        // stream.online
        self.helix
            .create_eventsub_subscription(
                StreamOnlineV1::broadcaster_user_id(channel_id),
                transport.clone(),
                &*token,
            )
            .await
            .map_err(|e| format!("Failed to subscribe stream.online: {e}"))?;
        info!("Subscribed to stream.online");

        // stream.offline
        self.helix
            .create_eventsub_subscription(
                StreamOfflineV1::broadcaster_user_id(channel_id),
                transport.clone(),
                &*token,
            )
            .await
            .map_err(|e| format!("Failed to subscribe stream.offline: {e}"))?;
        info!("Subscribed to stream.offline");

        // channel.update (v2)
        self.helix
            .create_eventsub_subscription(
                ChannelUpdateV2::broadcaster_user_id(channel_id),
                transport,
                &*token,
            )
            .await
            .map_err(|e| format!("Failed to subscribe channel.update: {e}"))?;
        info!("Subscribed to channel.update");

        Ok(())
    }
}

// ─── Notification State ──────────────────────────────────────────────

struct LiveState {
    go_live_message_id: Option<MessageId>,
    guild_id: Option<GuildId>,
}

// ─── Channel Lock/Unlock ─────────────────────────────────────────────

async fn set_channel_locked(ctx: &SerenityContext, channel_id: ChannelId, locked: bool) {
    let guild_id = match channel_id.to_channel(&ctx.http).await {
        Ok(channel) => match channel.guild() {
            Some(gc) => gc.guild_id,
            None => return,
        },
        Err(e) => {
            error!(error = %e, "Failed to fetch channel for lock/unlock");
            return;
        }
    };

    let everyone_role = RoleId::new(guild_id.get()); // @everyone role ID == guild ID

    let overwrite = PermissionOverwrite {
        allow: if locked {
            Permissions::empty()
        } else {
            Permissions::SEND_MESSAGES
        },
        deny: if locked {
            Permissions::SEND_MESSAGES
        } else {
            Permissions::empty()
        },
        kind: PermissionOverwriteType::Role(everyone_role),
    };

    let edit = EditChannel::new().permissions(vec![overwrite]);

    if let Err(e) = channel_id.edit(&ctx.http, edit).await {
        error!(error = %e, locked, "Failed to lock/unlock channel");
    } else {
        info!(channel_id = %channel_id, locked, "Channel lock state updated");
    }
}

// ─── Go-Live / Offline Handlers ──────────────────────────────────────

async fn handle_stream_online(
    ctx: &SerenityContext,
    twitch: &TwitchState,
    live_state: &Arc<RwLock<LiveState>>,
) {
    let stream = match twitch.fetch_stream_info().await {
        Ok(Some(s)) => s,
        Ok(None) => {
            warn!("stream.online received but no stream data from Helix");
            return;
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch stream info");
            return;
        }
    };

    let thumbnail = stream
        .thumbnail_url
        .replace("{width}", "440")
        .replace("{height}", "248");

    let twitch_url = format!(
        "https://twitch.tv/{}",
        std::env::var("TWITCH_USERNAME").unwrap_or_else(|_| "0xDC143C".into())
    );

    let embed = embeds::twitch_embed()
        .title(format!("LIVE: {}", stream.title))
        .url(&twitch_url)
        .field("Game", &stream.game_name, true)
        .field("Viewers", stream.viewer_count.to_string(), true)
        .image(&thumbnail);

    let content = twitch
        .config
        .live_role_id
        .map(|r| r.mention().to_string())
        .unwrap_or_default();

    let message = CreateMessage::new().content(&content).embed(embed.clone());

    match twitch
        .config
        .live_channel_id
        .send_message(&ctx.http, message)
        .await
    {
        Ok(msg) => {
            let mut state = live_state.write().await;
            state.go_live_message_id = Some(msg.id);
            state.guild_id = msg.guild_id;
            info!(message_id = %msg.id, "Go-live notification posted");
        }
        Err(e) => {
            error!(error = %e, "Failed to post go-live notification");
        }
    }

    // Unlock #live-chat
    if let Some(chat_channel) = twitch.config.live_chat_channel_id {
        set_channel_locked(ctx, chat_channel, false).await;
    }

    // Update bot status
    match serenity::all::ActivityData::streaming(&stream.title, &twitch_url) {
        Ok(activity) => ctx.set_activity(Some(activity)),
        Err(e) => error!(error = %e, "Failed to set streaming activity"),
    }
}

async fn handle_stream_offline(
    ctx: &SerenityContext,
    twitch: &TwitchState,
    live_state: &Arc<RwLock<LiveState>>,
) {
    let state = live_state.read().await;

    if let Some(msg_id) = state.go_live_message_id {
        let embed = embeds::twitch_embed()
            .title("STREAM ENDED")
            .description("Thanks for watching! See you next time.");

        let edit = EditMessage::new().embed(embed);
        if let Err(e) = twitch
            .config
            .live_channel_id
            .edit_message(&ctx.http, msg_id, edit)
            .await
        {
            error!(error = %e, "Failed to edit go-live message");
        } else {
            info!("Go-live message updated to STREAM ENDED");
        }
    }

    drop(state);

    {
        let mut state = live_state.write().await;
        state.go_live_message_id = None;
    }

    // Lock #live-chat
    if let Some(chat_channel) = twitch.config.live_chat_channel_id {
        set_channel_locked(ctx, chat_channel, true).await;
    }

    // Reset bot status
    ctx.set_activity(Some(serenity::all::ActivityData::watching(
        "the crimson tide",
    )));
}

async fn handle_channel_update(
    ctx: &SerenityContext,
    twitch: &TwitchState,
    live_state: &Arc<RwLock<LiveState>>,
    title: &str,
    category_name: &str,
) {
    let state = live_state.read().await;
    let msg_id = match state.go_live_message_id {
        Some(id) => id,
        None => return, // Not currently live, ignore
    };
    drop(state);

    let twitch_url = format!(
        "https://twitch.tv/{}",
        std::env::var("TWITCH_USERNAME").unwrap_or_else(|_| "0xDC143C".into())
    );

    // Re-fetch for updated viewer count + thumbnail
    let (viewers, thumbnail) = match twitch.fetch_stream_info().await {
        Ok(Some(s)) => (
            s.viewer_count,
            s.thumbnail_url
                .replace("{width}", "440")
                .replace("{height}", "248"),
        ),
        _ => (0, String::new()),
    };

    let mut embed = embeds::twitch_embed()
        .title(format!("LIVE: {title}"))
        .url(&twitch_url)
        .field("Game", category_name, true)
        .field("Viewers", viewers.to_string(), true);

    if !thumbnail.is_empty() {
        embed = embed.image(&thumbnail);
    }

    let edit = EditMessage::new().embed(embed);
    if let Err(e) = twitch
        .config
        .live_channel_id
        .edit_message(&ctx.http, msg_id, edit)
        .await
    {
        error!(error = %e, "Failed to update go-live embed");
    } else {
        info!(
            title,
            category_name, "Go-live embed updated (channel.update)"
        );
    }

    // Update bot status with new title
    match serenity::all::ActivityData::streaming(title, &twitch_url) {
        Ok(activity) => ctx.set_activity(Some(activity)),
        Err(e) => error!(error = %e, "Failed to set streaming activity"),
    }
}

// ─── Main EventSub Loop ─────────────────────────────────────────────

pub async fn start_eventsub(ctx: SerenityContext, twitch_config: TwitchConfig) {
    let twitch = match TwitchState::new(&twitch_config).await {
        Ok(t) => t,
        Err(e) => {
            error!(error = %e, "Failed to initialize Twitch client");
            return;
        }
    };

    twitch.spawn_refresh_loop();

    let live_state = Arc::new(RwLock::new(LiveState {
        go_live_message_id: None,
        guild_id: None,
    }));

    let mut url = TWITCH_EVENTSUB_URL.to_string();
    loop {
        match run_eventsub_connection(&ctx, &twitch, &live_state, &url).await {
            Ok(Some(reconnect_url)) => {
                info!("Reconnecting to new EventSub URL...");
                url = reconnect_url;
            }
            Ok(None) => {
                info!("EventSub connection closed cleanly, reconnecting in 5s...");
                url = TWITCH_EVENTSUB_URL.to_string();
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
            Err(e) => {
                error!(error = %e, "EventSub connection error, reconnecting in 5s...");
                url = TWITCH_EVENTSUB_URL.to_string();
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

/// Returns Ok(Some(url)) for reconnect, Ok(None) for clean close, Err for error.
async fn run_eventsub_connection(
    ctx: &SerenityContext,
    twitch: &TwitchState,
    live_state: &Arc<RwLock<LiveState>>,
    url: &str,
) -> Result<Option<String>, String> {
    let (ws_stream, _) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|e| format!("WebSocket connect failed: {e}"))?;

    info!("Connected to Twitch EventSub WebSocket");

    let (mut write, mut read) = ws_stream.split();
    let mut keepalive_timeout = std::time::Duration::from_secs(30);

    while let Ok(msg) = tokio::time::timeout(keepalive_timeout, read.next()).await {
        let text = match msg {
            Some(Ok(WsMessage::Text(text))) => text,
            Some(Ok(WsMessage::Close(_))) => {
                info!("EventSub WebSocket closed by server");
                return Ok(None);
            }
            Some(Ok(WsMessage::Ping(data))) => {
                let _ = write.send(WsMessage::Pong(data)).await;
                continue;
            }
            Some(Ok(_)) => continue,
            Some(Err(e)) => return Err(format!("WebSocket read error: {e}")),
            None => return Err("WebSocket stream ended".into()),
        };

        let parsed = match Event::parse_websocket(&text) {
            Ok(data) => data,
            Err(e) => {
                warn!(error = %e, "Failed to parse EventSub message");
                continue;
            }
        };

        match parsed {
            EventsubWebsocketData::Welcome { payload, .. } => {
                let session_id = payload.session.id.as_ref();

                if let Some(timeout_secs) = payload.session.keepalive_timeout_seconds {
                    keepalive_timeout = std::time::Duration::from_secs((timeout_secs as u64) + 5);
                }

                info!(session_id, "EventSub session established");

                if let Err(e) = twitch.subscribe_events(session_id).await {
                    error!(error = %e, "Failed to subscribe to events");
                }
            }

            EventsubWebsocketData::Keepalive { .. } => {
                // Server is alive, no action needed
            }

            EventsubWebsocketData::Notification { payload, .. } => match payload {
                Event::StreamOnlineV1(Payload {
                    message: Message::Notification(_notif),
                    ..
                }) => {
                    info!("Stream went live!");
                    handle_stream_online(ctx, twitch, live_state).await;
                }
                Event::StreamOfflineV1(Payload {
                    message: Message::Notification(_notif),
                    ..
                }) => {
                    info!("Stream went offline");
                    handle_stream_offline(ctx, twitch, live_state).await;
                }
                Event::ChannelUpdateV2(Payload {
                    message: Message::Notification(notif),
                    ..
                }) => {
                    handle_channel_update(
                        ctx,
                        twitch,
                        live_state,
                        &notif.title,
                        &notif.category_name,
                    )
                    .await;
                }
                other => {
                    warn!(event = ?other, "Unhandled EventSub notification");
                }
            },

            EventsubWebsocketData::Reconnect { payload, .. } => {
                if let Some(reconnect_url) = payload.session.reconnect_url.as_deref() {
                    info!(url = reconnect_url, "EventSub requesting reconnect");
                    return Ok(Some(reconnect_url.to_string()));
                }
                return Ok(None);
            }

            EventsubWebsocketData::Revocation { metadata, .. } => {
                warn!(
                    subscription_type = %metadata.subscription_type,
                    "EventSub subscription revoked"
                );
            }

            _ => {}
        }
    }

    Err("Keepalive timeout — server stopped responding".into())
}

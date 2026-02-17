use crate::config::TwitchConfig;
use crate::utils::embeds;
use futures_util::{SinkExt, StreamExt};
use serenity::all::{
    ChannelId, Context as SerenityContext, CreateMessage, EditChannel, EditMessage,
    Mentionable, MessageId, PermissionOverwrite, PermissionOverwriteType, Permissions, RoleId,
};
use serenity::model::id::GuildId;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{error, info, warn};

const TWITCH_EVENTSUB_URL: &str = "wss://eventsub.wss.twitch.tv/ws";
const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
const TWITCH_HELIX_URL: &str = "https://api.twitch.tv/helix";

// ─── OAuth2 Token Management ─────────────────────────────────────────

#[derive(Debug, Clone, serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Clone)]
struct TwitchAuth {
    client_id: String,
    client_secret: String,
    token: Arc<RwLock<String>>,
    http: reqwest::Client,
}

impl TwitchAuth {
    async fn new(client_id: String, client_secret: String) -> Result<Self, String> {
        let http = reqwest::Client::new();
        let auth = Self {
            client_id,
            client_secret,
            token: Arc::new(RwLock::new(String::new())),
            http,
        };
        auth.refresh_token().await?;
        Ok(auth)
    }

    async fn refresh_token(&self) -> Result<u64, String> {
        let resp = self
            .http
            .post(TWITCH_TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await
            .map_err(|e| format!("Token request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Token request returned {status}: {body}"));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {e}"))?;

        let expires_in = token_resp.expires_in;
        *self.token.write().await = token_resp.access_token;
        info!(expires_in, "Twitch access token refreshed");
        Ok(expires_in)
    }

    async fn get_token(&self) -> String {
        self.token.read().await.clone()
    }

    fn spawn_refresh_loop(self) {
        tokio::spawn(async move {
            loop {
                // Refresh 5 minutes before expiry, default to 1 hour
                let sleep_secs = match self.refresh_token().await {
                    Ok(expires_in) => expires_in.saturating_sub(300).max(60),
                    Err(e) => {
                        error!(error = %e, "Failed to refresh Twitch token, retrying in 60s");
                        60
                    }
                };
                tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)).await;
            }
        });
    }
}

// ─── Helix API ───────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct HelixResponse<T> {
    data: Vec<T>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct StreamData {
    title: String,
    game_name: String,
    viewer_count: u64,
    thumbnail_url: String,
    started_at: String,
}

async fn fetch_stream_info(
    auth: &TwitchAuth,
    channel_id: &str,
) -> Result<Option<StreamData>, String> {
    let token = auth.get_token().await;
    let resp = auth
        .http
        .get(format!("{TWITCH_HELIX_URL}/streams"))
        .query(&[("user_id", channel_id)])
        .header("Client-Id", &auth.client_id)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Helix request failed: {e}"))?;

    let data: HelixResponse<StreamData> = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse stream data: {e}"))?;

    Ok(data.data.into_iter().next())
}

// ─── EventSub WebSocket ──────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct EventSubMessage {
    metadata: EventSubMetadata,
    payload: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
struct EventSubMetadata {
    message_type: String,
    #[serde(default)]
    subscription_type: Option<String>,
}

async fn subscribe_to_event(
    auth: &TwitchAuth,
    session_id: &str,
    event_type: &str,
    channel_id: &str,
) -> Result<(), String> {
    let token = auth.get_token().await;

    let condition = match event_type {
        "stream.online" | "stream.offline" => {
            serde_json::json!({ "broadcaster_user_id": channel_id })
        }
        "channel.update" => {
            serde_json::json!({ "broadcaster_user_id": channel_id })
        }
        _ => return Err(format!("Unknown event type: {event_type}")),
    };

    let version = match event_type {
        "channel.update" => "2",
        _ => "1",
    };

    let body = serde_json::json!({
        "type": event_type,
        "version": version,
        "condition": condition,
        "transport": {
            "method": "websocket",
            "session_id": session_id,
        }
    });

    let resp = auth
        .http
        .post(format!("{TWITCH_HELIX_URL}/eventsub/subscriptions"))
        .header("Client-Id", &auth.client_id)
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("EventSub subscribe failed: {e}"))?;

    if resp.status().is_success() {
        info!(event_type, "Subscribed to EventSub event");
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(format!("EventSub subscribe returned {status}: {body}"))
    }
}

// ─── Notification State ──────────────────────────────────────────────

struct LiveState {
    go_live_message_id: Option<MessageId>,
    guild_id: Option<GuildId>,
}

// ─── Channel Lock/Unlock ─────────────────────────────────────────────

async fn set_channel_locked(
    ctx: &SerenityContext,
    channel_id: ChannelId,
    locked: bool,
) {
    // Get guild ID from the channel
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
    auth: &TwitchAuth,
    twitch_config: &TwitchConfig,
    live_state: &Arc<RwLock<LiveState>>,
) {
    let stream = match fetch_stream_info(auth, &twitch_config.channel_id).await {
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

    let content = twitch_config
        .live_role_id
        .map(|r| r.mention().to_string())
        .unwrap_or_default();

    let message = CreateMessage::new().content(&content).embed(embed.clone());

    match twitch_config
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
    if let Some(chat_channel) = twitch_config.live_chat_channel_id {
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
    twitch_config: &TwitchConfig,
    live_state: &Arc<RwLock<LiveState>>,
) {
    let state = live_state.read().await;

    // Edit the go-live embed to show STREAM ENDED
    if let Some(msg_id) = state.go_live_message_id {
        let embed = embeds::twitch_embed()
            .title("STREAM ENDED")
            .description("Thanks for watching! See you next time.");

        let edit = EditMessage::new().embed(embed);
        if let Err(e) = twitch_config
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

    // Clear stored message ID
    {
        let mut state = live_state.write().await;
        state.go_live_message_id = None;
    }

    // Lock #live-chat
    if let Some(chat_channel) = twitch_config.live_chat_channel_id {
        set_channel_locked(ctx, chat_channel, true).await;
    }

    // Reset bot status
    ctx.set_activity(Some(serenity::all::ActivityData::watching(
        "the crimson tide",
    )));
}

async fn handle_channel_update(
    ctx: &SerenityContext,
    auth: &TwitchAuth,
    twitch_config: &TwitchConfig,
    live_state: &Arc<RwLock<LiveState>>,
    payload: &serde_json::Value,
) {
    let state = live_state.read().await;
    let msg_id = match state.go_live_message_id {
        Some(id) => id,
        None => return, // Not currently live, ignore
    };
    drop(state);

    let title = payload["event"]["title"]
        .as_str()
        .unwrap_or("Untitled Stream");
    let game = payload["event"]["category_name"]
        .as_str()
        .unwrap_or("Unknown");

    let twitch_url = format!(
        "https://twitch.tv/{}",
        std::env::var("TWITCH_USERNAME").unwrap_or_else(|_| "0xDC143C".into())
    );

    // Re-fetch for updated viewer count + thumbnail
    let (viewers, thumbnail) = match fetch_stream_info(auth, &twitch_config.channel_id).await {
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
        .field("Game", game, true)
        .field("Viewers", viewers.to_string(), true);

    if !thumbnail.is_empty() {
        embed = embed.image(&thumbnail);
    }

    let edit = EditMessage::new().embed(embed);
    if let Err(e) = twitch_config
        .live_channel_id
        .edit_message(&ctx.http, msg_id, edit)
        .await
    {
        error!(error = %e, "Failed to update go-live embed");
    } else {
        info!(title, game, "Go-live embed updated (channel.update)");
    }

    // Update bot status with new title
    match serenity::all::ActivityData::streaming(title, &twitch_url) {
        Ok(activity) => ctx.set_activity(Some(activity)),
        Err(e) => error!(error = %e, "Failed to set streaming activity"),
    }
}

// ─── Main EventSub Loop ─────────────────────────────────────────────

pub async fn start_eventsub(ctx: SerenityContext, twitch_config: TwitchConfig) {
    let auth = match TwitchAuth::new(
        twitch_config.client_id.clone(),
        twitch_config.client_secret.clone(),
    )
    .await
    {
        Ok(a) => a,
        Err(e) => {
            error!(error = %e, "Failed to initialize Twitch auth");
            return;
        }
    };

    // Spawn token refresh loop
    auth.clone().spawn_refresh_loop();

    let live_state = Arc::new(RwLock::new(LiveState {
        go_live_message_id: None,
        guild_id: None,
    }));

    loop {
        if let Err(e) =
            run_eventsub_connection(&ctx, &auth, &twitch_config, &live_state).await
        {
            error!(error = %e, "EventSub connection error, reconnecting in 5s...");
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn run_eventsub_connection(
    ctx: &SerenityContext,
    auth: &TwitchAuth,
    twitch_config: &TwitchConfig,
    live_state: &Arc<RwLock<LiveState>>,
) -> Result<(), String> {
    let (ws_stream, _) = tokio_tungstenite::connect_async(TWITCH_EVENTSUB_URL)
        .await
        .map_err(|e| format!("WebSocket connect failed: {e}"))?;

    info!("Connected to Twitch EventSub WebSocket");

    let (mut write, mut read) = ws_stream.split();
    let mut keepalive_timeout = std::time::Duration::from_secs(30);

    while let Ok(msg) = tokio::time::timeout(keepalive_timeout, read.next()).await {
        let msg = match msg {
            Some(Ok(WsMessage::Text(text))) => text,
            Some(Ok(WsMessage::Close(_))) => {
                info!("EventSub WebSocket closed by server");
                return Ok(());
            }
            Some(Ok(WsMessage::Ping(data))) => {
                let _ = write.send(WsMessage::Pong(data)).await;
                continue;
            }
            Some(Ok(_)) => continue,
            Some(Err(e)) => return Err(format!("WebSocket read error: {e}")),
            None => return Err("WebSocket stream ended".into()),
        };

        let parsed: EventSubMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                warn!(error = %e, "Failed to parse EventSub message");
                continue;
            }
        };

        match parsed.metadata.message_type.as_str() {
            "session_welcome" => {
                let session_id = parsed.payload["session"]["id"]
                    .as_str()
                    .unwrap_or_default();

                if let Some(timeout_secs) =
                    parsed.payload["session"]["keepalive_timeout_seconds"].as_u64()
                {
                    // Add buffer to keepalive timeout
                    keepalive_timeout =
                        std::time::Duration::from_secs(timeout_secs + 5);
                }

                info!(session_id, "EventSub session established");

                // Subscribe to events
                for event in &["stream.online", "stream.offline", "channel.update"] {
                    if let Err(e) = subscribe_to_event(
                        auth,
                        session_id,
                        event,
                        &twitch_config.channel_id,
                    )
                    .await
                    {
                        error!(event, error = %e, "Failed to subscribe to event");
                    }
                }
            }

            "session_keepalive" => {
                // Server is alive, no action needed
            }

            "notification" => {
                let sub_type = parsed
                    .metadata
                    .subscription_type
                    .as_deref()
                    .unwrap_or_default();

                match sub_type {
                    "stream.online" => {
                        info!("Stream went live!");
                        handle_stream_online(ctx, auth, twitch_config, live_state).await;
                    }
                    "stream.offline" => {
                        info!("Stream went offline");
                        handle_stream_offline(ctx, twitch_config, live_state).await;
                    }
                    "channel.update" => {
                        handle_channel_update(
                            ctx,
                            auth,
                            twitch_config,
                            live_state,
                            &parsed.payload,
                        )
                        .await;
                    }
                    other => {
                        warn!(event_type = other, "Unhandled EventSub notification");
                    }
                }
            }

            "session_reconnect" => {
                let reconnect_url = parsed.payload["session"]["reconnect_url"]
                    .as_str()
                    .unwrap_or_default();
                info!(url = reconnect_url, "EventSub requesting reconnect");
                // Let the outer loop reconnect (could use reconnect_url in future)
                return Ok(());
            }

            "revocation" => {
                warn!(
                    reason = parsed.payload["subscription"]["status"].as_str().unwrap_or("unknown"),
                    "EventSub subscription revoked"
                );
            }

            other => {
                warn!(message_type = other, "Unknown EventSub message type");
            }
        }
    }

    Err("Keepalive timeout — server stopped responding".into())
}

use std::time::Duration;

use axum::http::HeaderValue;
use serenity::{
    async_trait,
    framework::{
        standard::macros::group,
        StandardFramework,
    },
    json::Value,
    model::{
        gateway::Ready,
        prelude::Activity,
    },
    prelude::*,
};
use tokio::sync::broadcast::Receiver;

use crate::{
    config::DiscordBotArgs,
    db,
    discord::client::{
        DiscordClient,
        DiscordOAuth2Client,
    },
    tautulli::client::TautulliClient,
};

mod channel_statistics;
mod commands;
mod usermeta_refresh;

use self::{
    channel_statistics::ChannelStatisticArgs,
    commands::*,
    usermeta_refresh::UserMetadataRefreshArgs,
};

struct Handler {
    config: DiscordBotArgs,
}

#[group]
#[commands(ping)]
struct General;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        ctx.set_activity(Activity::watching(&self.config.discord_bot_status))
            .await;
    }

    async fn unknown(&self, _ctx: Context, _name: String, _raw: Value) {
        println!("unknown")
    }
}

pub async fn run(mut kill: Receiver<()>, config: DiscordBotArgs) {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = StandardFramework::new().group(&GENERAL_GROUP);

    let mut client = Client::builder(
        &config.discord.discord_bot_token.sensitive_string(),
        intents,
    )
    .event_handler(Handler {
        config: config.clone(),
    })
    .framework(framework)
    .await
    .expect("Err creating client");

    let manager = client.shard_manager.clone();
    let stat_kill = kill.resubscribe();
    let meta_kill = kill.resubscribe();
    tokio::spawn(async move {
        tokio::select! {
            _ = kill.recv() => tracing::info!("shutting down bot..."),
        }
        let mut lock = manager.lock().await;
        lock.shutdown_all().await;
    });

    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.append("Accept", HeaderValue::from_static("application/json"));

    let reqwest_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .default_headers(default_headers)
        .build()
        .unwrap();

    let tautulli_client = TautulliClient::new(
        &reqwest_client,
        &config.tautulli.tautulli_url,
        &config.tautulli.tautulli_api_key.sensitive_string(),
    );

    let pool = db::initialize_db_pool(&config.database.database_url.sensitive_string())
        .await
        .unwrap();

    let discord_oauth_client = DiscordOAuth2Client::new(
        reqwest_client.clone(),
        &config.discord.discord_client_id.sensitive_string(),
        &config.discord.discord_client_secret.sensitive_string(),
        None,
    );

    let discord_client = DiscordClient::new(
        reqwest_client.clone(),
        &config.discord.discord_bot_token.sensitive_string(),
    );

    usermeta_refresh::setup(
        meta_kill,
        UserMetadataRefreshArgs {
            application_name: config.application_name,
            client_id: config.discord.discord_client_id.sensitive_string(),
            update_interval: config.discord_user_update_interval.into(),
            pool: pool.clone(),
            discord_client: discord_client.clone(),
            discord_oauth_client: discord_oauth_client.clone(),
            tautulli_client: tautulli_client.clone(),
        },
    )
    .await;

    channel_statistics::setup(
        stat_kill,
        ChannelStatisticArgs {
            tautulli_client,
            interval_seconds: config.discord_stat_update_interval.into(),
            cache_and_http_client: client.cache_and_http.clone(),
            config: config.channel_config,
            server_id: config.discord.discord_server_id,
        },
    )
    .await
    .unwrap();

    client.start().await.unwrap();
}

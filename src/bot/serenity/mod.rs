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
    tautulli::client::TautulliClient,
};

mod channel_statistics;
mod commands;

use self::commands::*;

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

    let mut client = Client::builder(&config.discord_bot_token.sensitive_string(), intents)
        .event_handler(Handler {
            config: config.clone(),
        })
        .framework(framework)
        .await
        .expect("Err creating client");

    let manager = client.shard_manager.clone();
    let stat_kill = kill.resubscribe();
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

    channel_statistics::setup(
        stat_kill,
        config.discord_stat_update_interval.into(),
        client.cache_and_http.clone(),
        tautulli_client,
        config.channel_config,
    )
    .await
    .unwrap();

    client.start().await.unwrap();
}

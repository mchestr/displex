use std::time::Duration;

use anyhow::Result;
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
    config::DisplexConfig,
    db,
    discord::client::{
        DiscordClient,
        DiscordOAuth2Client,
    },
    tautulli::client::TautulliClient, utils::DisplexClients,
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
    config: DisplexConfig,
}

#[group]
#[commands(ping)]
struct General;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        ctx.set_activity(Activity::watching(&self.config.discord_bot.status_text))
            .await;
    }

    async fn unknown(&self, _ctx: Context, _name: String, _raw: Value) {
        println!("unknown")
    }
}

pub async fn init(config: DisplexConfig) -> Result<serenity::Client> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = StandardFramework::new().group(&GENERAL_GROUP);

    Ok(Client::builder(&config.discord_bot.token, intents)
        .event_handler(Handler {
            config: config.clone(),
        })
        .framework(framework)
        .await?)
}

pub async fn run(mut kill: Receiver<()>, config: DisplexConfig, mut serenity_client: serenity::Client, clients: &DisplexClients) {

    let manager = serenity_client.shard_manager.clone();
    let stat_kill = kill.resubscribe();
    let meta_kill = kill.resubscribe();
    tokio::spawn(async move {
        tokio::select! {
            _ = kill.recv() => tracing::info!("shutting down bot..."),
        }
        let mut lock = manager.lock().await;
        lock.shutdown_all().await;
    });

    if let Some(job_config) = config.discord_bot.user_update {
        usermeta_refresh::setup(
            meta_kill,
            UserMetadataRefreshArgs {
                application_name: config.application_name,
                client_id: config.discord.client_id,
                update_interval: job_config.interval,
                pool: clients.pool.clone(),
                discord_client: clients.discord_client.clone(),
                discord_oauth_client: clients.discord_oauth2_client.clone(),
                tautulli_client: clients.tautulli_client.clone(),
            },
        )
        .await;
    }

    if let Some(job_config) = config.discord_bot.stat_update {
        channel_statistics::setup(
            stat_kill,
            ChannelStatisticArgs {
                tautulli_client: clients.tautulli_client.clone(),
                interval_seconds: job_config.interval,
                http_client: clients.serenity_client.clone(),
                config: job_config,
                server_id: config.discord.server_id,
            },
        )
        .await
        .unwrap();
    }

    serenity_client.start().await.unwrap();
}

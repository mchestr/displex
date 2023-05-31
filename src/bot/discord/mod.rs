use anyhow::Result;

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
    config::AppConfig,
    services::AppServices,
};

mod channel_statistics;
mod commands;
mod usermeta_refresh;

use self::commands::*;

struct Handler {
    config: AppConfig,
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

pub async fn init(config: AppConfig) -> Result<serenity::Client> {
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

pub async fn run(
    mut kill: Receiver<()>,
    config: &AppConfig,
    mut serenity_client: serenity::Client,
    services: &AppServices,
) {
    let manager = serenity_client.shard_manager.clone();
    let _stat_kill = kill.resubscribe();
    let meta_kill = kill.resubscribe();
    tokio::spawn(async move {
        tokio::select! {
            _ = kill.recv() => tracing::info!("shutting down bot..."),
        }
        let mut lock = manager.lock().await;
        lock.shutdown_all().await;
    });

    if config.discord_bot.user_update.enabled {
        usermeta_refresh::setup(
            meta_kill,
            config,
            services,
        )
        .await;
    }

    // if let Some(job_config) = config.discord_bot.stat_update {
    //     channel_statistics::setup(
    //         stat_kill,
    //         ChannelStatisticArgs {
    //             tautulli_client: clients.tautulli_client.clone(),
    //             interval_seconds: job_config.interval,
    //             http_client: clients.serenity_client.clone(),
    //             config: job_config,
    //             server_id: config.discord.server_id,
    //         },
    //     )
    //     .await
    //     .unwrap();
    // }

    serenity_client.start().await.unwrap();
}

use anyhow::Result;

use serenity::{
    async_trait,
    client::ClientBuilder,
    framework::{
        standard::macros::group,
        StandardFramework,
    },
    http::Http,
    json::Value,
    model::{
        gateway::Ready,
        prelude::Activity,
    },
    prelude::*,
};
use tokio::sync::broadcast::Receiver;

use crate::config::AppConfig;

pub mod channel_statistics;
mod commands;
pub mod usermeta_refresh;

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

pub async fn init(config: AppConfig, client: Http) -> Result<serenity::Client> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = StandardFramework::new().group(&GENERAL_GROUP);

    Ok(ClientBuilder::new_with_http(client, intents)
        .event_handler(Handler {
            config: config.clone(),
        })
        .framework(framework)
        .await?)
}

pub async fn run(mut kill: Receiver<()>, mut serenity_client: serenity::Client) {
    let manager = serenity_client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = kill.recv() => tracing::info!("shutting down bot..."),
        }
        let mut lock = manager.lock().await;
        lock.shutdown_all().await;
    });
    serenity_client.start().await.unwrap();
}

use std::collections::HashSet;

use anyhow::Result;

use serenity::{
    async_trait,
    client::ClientBuilder,
    framework::{
        standard::{
            macros::group,
            Configuration,
        },
        StandardFramework,
    },
    gateway::ActivityData,
    http::HttpBuilder,
    model::gateway::Ready,
    prelude::*,
};
use tokio::sync::broadcast::Receiver;

use crate::{
    config::AppConfig,
    services::AppServices,
};

mod commands;

use self::commands::*;

struct Handler {
    config: AppConfig,
}

#[group]
#[commands(ping, status)]
struct General;

#[group]
#[prefix = "subs"]
#[commands(subscriber_tokens)]
struct Subscribers;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        ctx.set_activity(Some(ActivityData::watching(
            &self.config.discord_bot.status_text,
        )));
    }
}

pub async fn init(config: AppConfig, services: &AppServices) -> Result<serenity::Client> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let http_client = HttpBuilder::new(&config.discord_bot.token)
        .client(services.reqwest_client.clone())
        .build();

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http_client.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .group(&GENERAL_GROUP)
        .group(&SUBSCRIBERS_GROUP);
    framework.configure(Configuration::new().owners(owners).prefix("~"));

    let client = ClientBuilder::new_with_http(http_client, intents)
        .event_handler(Handler {
            config: config.clone(),
        })
        .framework(framework)
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<AppServices>(services.clone());
    }

    Ok(client)
}

pub async fn run(mut kill: Receiver<()>, mut serenity_client: serenity::Client) {
    let manager = serenity_client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = kill.recv() => tracing::info!("shutting down bot..."),
        }
        manager.shutdown_all().await;
    });
    serenity_client.start().await.unwrap();
}

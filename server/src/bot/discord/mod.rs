use std::collections::HashSet;

use anyhow::Result;

use serenity::{
    client::ClientBuilder,
    http::HttpBuilder,
    prelude::*,
};
use tokio::sync::broadcast::Receiver;

use crate::{
    config::AppConfig,
    services::AppServices,
};

mod commands;

pub async fn init(config: AppConfig, services: &AppServices) -> Result<serenity::Client> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let http_client = HttpBuilder::new(&config.discord_bot.token)
        .client(services.reqwest_client.clone())
        .build();

    // We will fetch your bot's owners and id
    let (_owners, _bot_id) = match http_client.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::ping(),
            commands::subscriber_tokens(),
            commands::stats(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            ..Default::default()
        },
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        ..Default::default()
    };
    let services = services.clone();
    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                // Set the bot's activity status
                ctx.set_activity(Some(serenity::all::ActivityData::watching(
                    config.discord_bot.status_text,
                )));

                Ok(services)
            })
        })
        .options(options)
        .build();

    let client = ClientBuilder::new_with_http(http_client, intents)
        .framework(framework)
        .await?;

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

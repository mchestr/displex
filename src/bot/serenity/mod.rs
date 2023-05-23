use std::time::Duration;

use axum::http::HeaderValue;
use serenity::{
    async_trait,
    framework::{
        standard::{
            macros::{
                command,
                group,
            },
            CommandResult,
        },
        StandardFramework,
    },
    json::Value,
    model::{
        channel::Message,
        gateway::Ready,
        prelude::Activity,
    },
    prelude::*,
};
use tokio::sync::broadcast::Receiver;


use crate::{
    config::{DiscordBotArgs},
    tautulli::client::TautulliClient,
};

mod channel_statistics;

struct Handler {
    config: DiscordBotArgs,
}

#[group]
#[commands(ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
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
        config.discord_stat_update_interval.as_secs().try_into().unwrap(),
        client.cache_and_http.clone(),
        tautulli_client,
        config.channel_config,
    )
    .await
    .unwrap();


    client.start().await.unwrap();
}

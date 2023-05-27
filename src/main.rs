use std::time::Duration;

use anyhow::{
    Context,
    Result,
};
use axum::http::HeaderValue;
use clap::Parser;

use displex::{
    bot::DisplexBot,
    config::{self,},
    db,
    discord::client::{
        DiscordClient,
        DiscordOAuth2Client,
    },
    server::DisplexHttpServer,
    tautulli::client::TautulliClient, utils,
};
use tokio::signal::unix::{
    signal,
    SignalKind,
};
use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    fmt,
    prelude::*,
    EnvFilter,
};

#[derive(Parser)]
#[command(name = "displex")]
#[command(about = "A Discord/Plex/Tautulli Application", long_about = None)]
struct Cli {
    #[clap(short, long, default_value = "config.toml")]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => println!("loaded .env file."),
        Err(_) => println!("no .env file found."),
    };

    let rust_log = match std::option_env!("RUST_LOG") {
        Some(value) => value,
        None => {
            "displex=info,tower_http=info,axum::rejection=debug,h2=warn,serenity=info,reqwest=info"
        }
    };

    tracing_subscriber::registry()
        // Continue logging to stdout
        .with(
            fmt::Layer::default().with_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .parse_lossy(rust_log),
            ),
        )
        .try_init()
        .context("failed to initialize logging")?;

    let args = Cli::parse();
    let config = config::load(&args.config_file)?;
    tracing::info!("{:#?}", config);

    let (tx, rx) = tokio::sync::broadcast::channel::<()>(1);
    tokio::spawn(async move {
        let mut int = signal(SignalKind::interrupt()).expect("error");
        let mut term = signal(SignalKind::terminate()).expect("error");

        tokio::select! {
            _ = int.recv() => tracing::info!("sigint received"),
            _ = term.recv() => tracing::info!("sigterm received"),
        };
        tx.send(())
    });

    let (serenity_client, clients) = utils::initialize_clients(&config).await?;

    tokio::try_join!(
        config.http.type_.run(rx.resubscribe(), config.clone(), &clients),
        config.discord_bot.type_.run(rx, config.clone(), serenity_client, &clients)
    )?;
    Ok(())
}

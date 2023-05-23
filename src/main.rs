use clap::{
    Parser,
    Subcommand,
};
use derive_more::Display;
use displex::{
    bot::DisplexBot,
    config::{
        DiscordBotArgs,
        RefreshArgs,
        ServerArgs,
        SetMetadataArgs,
    },
    metadata,
    server::DisplexHttpServer,
    tasks,
};
use tokio::signal::unix::{
    signal,
    SignalKind,
};
use tracing_subscriber::{
    fmt,
    prelude::*,
    EnvFilter,
};

#[derive(Parser, Display)]
#[command(name = "displex")]
#[command(about = "A Discord/Plex/Tautulli Application", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Display)]
enum Commands {
    Server(ServerArgs),
    Refresh(RefreshArgs),
    SetMetadata(SetMetadataArgs),
    DiscordBot(DiscordBotArgs),
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => println!("loaded .env file."),
        Err(_) => println!("no .env file found."),
    };

    tracing_subscriber::registry()
        // Continue logging to stdout
        .with(fmt::Layer::default().with_filter(EnvFilter::from_default_env()))
        .try_init()
        .unwrap();

    let args = Cli::parse();
    tracing::info!("DisplexConfig({:#})", args);

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

    match args.command {
        Commands::Server(args) => args.http_server.run(rx, args.clone()).await,
        Commands::Refresh(args) => tasks::stat_refresh::run(args).await,
        Commands::SetMetadata(args) => metadata::set_metadata(args).await,
        Commands::DiscordBot(args) => args.discord_bot.run(rx, args.clone()).await,
    };
    Ok(())
}

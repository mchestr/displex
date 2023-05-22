use clap::{
    Parser,
    Subcommand,
};
use derive_more::Display;
use displex::{
    bot::{DisplexBot},
    config::{
        RefreshArgs,
        ServerArgs,
    },
    server::DisplexHttpServer,
    tasks,
};
use tokio::signal::unix::{SignalKind, signal};
use tracing_subscriber::{
    fmt,
    prelude::*,
    EnvFilter,
};

#[derive(Parser, Display)] // requires `derive` feature
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
    tracing::info!("DisplexConfig({})", args);

    let (tx, rx) = tokio::sync::broadcast::channel::<()>(1);
    let kill = tx.clone();
    tokio::spawn(async move {
        let mut int = signal(SignalKind::interrupt()).expect("error");
        let mut term = signal(SignalKind::terminate()).expect("error");

        tokio::select! {
            _ = int.recv() => tracing::info!("sigint received"),
            _ = term.recv() => tracing::info!("sigterm received"),
        };
        kill.send(())
    });

    match args.command {
        Commands::Server(args) => {
            let bot_kill = tx.subscribe();
            tokio::join!(
                args.http_server.run(rx, args.clone()), 
                args.discord.discord_bot.run(bot_kill, args.clone())
            );
        }
        Commands::Refresh(args) => {
            tokio::join!(tasks::stat_refresh::run(args));
        }
    };
    Ok(())
}

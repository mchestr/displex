use clap::{
    Parser,
    Subcommand,
};
use derive_more::Display;
use displex::{
    config::{
        RefreshArgs,
        ServerArgs,
    },
    server::DisplexHttpServer,
    tasks,
};
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

    match args.command {
        Commands::Server(args) => args.http_server.run(args.clone()).await,
        Commands::Refresh(args) => tasks::stat_refresh::run(args).await?,
    };
    Ok(())
}

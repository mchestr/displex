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
    server,
    tasks,
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
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let args = Cli::parse();
    tracing::info!("DisplexConfig({})", args);

    match args.command {
        Commands::Server(args) => server::run(args).await?,
        Commands::Refresh(args) => tasks::stat_refresh::run(args).await?,
    };
    Ok(())
}

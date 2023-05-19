
use clap::{Parser, Subcommand};
use displex::{config::{ServerArgs, RefreshArgs}, server, tasks};

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "displex")]
#[command(about = "A Discord/Plex/Tautulli Application", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Server(ServerArgs),
    Refresh(RefreshArgs),
}

fn main() -> std::io::Result<()> {
    dotenvy::dotenv().unwrap();
    let args = Cli::parse();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("{:#?}", args);

    Ok(match args.command {
        Commands::Server(args) => server::run(args),
        Commands::Refresh(args) => tasks::stat_refresh::run(args),
    })
}

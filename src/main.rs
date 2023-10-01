use anyhow::Result;

use clap::{
    Parser,
    Subcommand,
};

use displex::{
    bot::DisplexBot,
    config::{self,},
    graphql::get_schema,
    migrations::Migrator,
    server::DisplexHttpServer,
    services::create_app_services,
};
use sea_orm::{
    Database,
    DatabaseConnection,
};
use sea_orm_migration::MigratorTrait;

use tokio::signal::unix::{
    signal,
    SignalKind,
};

#[derive(Parser)]
#[command(name = "displex")]
#[command(about = "A Discord/Plex/Tautulli Application", long_about = None)]
struct Cli {
    #[clap(short, long, default_value = ".")]
    config_dir: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Bot,
    ChannelRefresh,
    Metadata,
    RequestsUpgrade,
    Server,
    UserRefresh,
}

#[tokio::main]
async fn main() -> Result<()> {
    if dotenvy::dotenv().is_err() {
        println!("no .env found.");
    }
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    let config = config::load(&args.config_dir)?;
    tracing::info!("{:#?}", config);

    let db = Database::connect(&config.database.url)
        .await
        .expect("Database connection failed");
    if !config.database.read_only {
        Migrator::up(&db, None).await?;
    }

    let selected_database = match db {
        DatabaseConnection::SqlxSqlitePoolConnection(_) => "SQLite",
        DatabaseConnection::SqlxMySqlPoolConnection(_) => "MySQL",
        DatabaseConnection::SqlxPostgresPoolConnection(_) => "PostgreSQL",
        _ => "Unrecognized",
    };
    tracing::info!("Using database backend: {selected_database:?}");

    let (serenity_client, app_services) = create_app_services(db.clone(), &config).await;
    let schema = get_schema(&app_services, db.clone(), &config).await;

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
        Commands::Bot => {
            config.discord_bot.type_.run(rx, serenity_client).await?;
        }
        Commands::ChannelRefresh => {
            displex::tasks::channel_statistics::run(&config, &app_services).await?;
        }
        Commands::Metadata => {
            displex::tasks::metadata::run(&config).await?;
        }
        Commands::RequestsUpgrade => {
            displex::tasks::requests_upgrade::run(&app_services).await?;
        }
        Commands::Server => {
            config
                .http
                .type_
                .run(rx, config.clone(), &app_services, &schema)
                .await?;
        }
        Commands::UserRefresh => {
            displex::tasks::user_refresh::run(&config, &app_services).await?;
        }
    }
    Ok(())
}

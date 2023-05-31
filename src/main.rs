use anyhow::{
    Context,
    Result,
};

use clap::Parser;

use displex::{
    bot::DisplexBot,
    config::{self,},
    server::DisplexHttpServer,
    utils,
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
    if dotenvy::dotenv().is_err() {
        println!("no .env found.");
    }
    tracing_subscriber::fmt::init();
    let config = config::get_app_config()?;
    tracing::info!("application config >\n{:#?}", config);

    let args = Cli::parse();
    let config = config::load(&args.config_file)?;
    tracing::info!("{:#?}", config);

    let db = Database::connect(&config.database.url)
        .await
        .expect("Database connection failed");
    Migrator::up(&db, None).await?;

    let selected_database = match db {
        DatabaseConnection::SqlxSqlitePoolConnection(_) => "SQLite",
        DatabaseConnection::SqlxMySqlPoolConnection(_) => "MySQL",
        DatabaseConnection::SqlxPostgresPoolConnection(_) => "PostgreSQL",
        _ => "Unrecognized",
    };
    tracing::info!("Using database backend: {selected_database:?}");

    let serenity_client = bot::discord::init(config.clone()).await?;
    let discord_http_client = serenity_client.cache_and_http.http.clone();
    let app_services = create_app_services(db.clone(), &config, &discord_http_client).await;
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

    let (serenity_client, clients) = utils::initialize_clients(&config).await?;

    tokio::try_join!(
        config
            .http
            .type_
            .run(rx.resubscribe(), config.clone(), &clients, &schema),
        config
            .discord_bot
            .type_
            .run(rx, config.clone(), serenity_client, &clients)
    )?;
    Ok(())
}

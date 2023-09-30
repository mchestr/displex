use std::time::Duration;

use anyhow::Result;
use axum::http::HeaderValue;
use clap::Parser;
use displex::{
    config::{
        self,
        AppConfig,
    },
    discord::models::ApplicationMetadataDefinition,
};

#[derive(Parser)]
#[command(name = "displex")]
#[command(about = "A Discord/Plex/Tautulli Application", long_about = None)]
struct Cli {
    #[clap(short, long, default_value = ".")]
    config_dir: String,
}

pub async fn set_metadata(config: AppConfig) {
    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.append("Accept", HeaderValue::from_static("application/json"));

    let reqwest_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .default_headers(default_headers)
        .danger_accept_invalid_certs(config.debug.accept_invalid_certs)
        .build()
        .unwrap();

    register_metadata(config, &reqwest_client).await.unwrap();
}

async fn register_metadata(config: AppConfig, client: &reqwest::Client) -> Result<()> {
    let metadata_spec = vec![
        ApplicationMetadataDefinition {
            key: "is_subscribed".into(),
            name: "⭐".into(),
            description: "Access to Plex Server".into(),
            type_: 7,
        },
        ApplicationMetadataDefinition {
            key: "watched_hours".into(),
            name: "Hours Streamed".into(),
            description: "Hours spent streaming".into(),
            type_: 2,
        },
    ];
    // let metadata_spec = vec![];

    let current_metadata: Vec<ApplicationMetadataDefinition> = client
        .get(format!(
            "https://discord.com/api/v10/applications/{}/role-connections/metadata",
            config.discord.client_id
        ))
        .header("Authorization", format!("Bot {}", config.discord_bot.token))
        .send()
        .await?
        .json()
        .await?;

    tracing::info!("Discord Metadata: {:#?}", current_metadata);
    if current_metadata != metadata_spec {
        tracing::info!("Registering Discord application metadata");
        tracing::info!(
            "{:#?}",
            client
                .put(format!(
                    "https://discord.com/api/v10/applications/{}/role-connections/metadata",
                    config.discord.client_id
                ))
                .header("Authorization", format!("Bot {}", config.discord_bot.token))
                .json(&metadata_spec)
                .send()
                .await?
                .json()
                .await?
        );
    } else {
        tracing::info!("Discord application metadata is up to date")
    }

    Ok(())
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

    set_metadata(config).await;
    Ok(())
}

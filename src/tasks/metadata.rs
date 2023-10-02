use std::time::Duration;

use crate::{
    config::AppConfig,
    services::discord::models::ApplicationMetadataDefinition,
};
use anyhow::Result;
use axum::http::HeaderValue;

pub async fn run(config: &AppConfig) -> Result<()> {
    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.append("Accept", HeaderValue::from_static("application/json"));

    let reqwest_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .default_headers(default_headers)
        .danger_accept_invalid_certs(config.debug.accept_invalid_certs)
        .build()?;

    register_metadata(config, &reqwest_client).await?;
    Ok(())
}

async fn register_metadata(config: &AppConfig, client: &reqwest::Client) -> Result<()> {
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
        tracing::info!("Discord application metadata is up to date");
    }

    Ok(())
}

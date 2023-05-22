use std::time::Duration;

use anyhow::Result;
use axum::http::HeaderValue;

use crate::{
    config::SetMetadataArgs,
    discord::{
        client::DiscordClient,
        models::ApplicationMetadataDefinition,
    },
};

pub async fn set_metadata(config: SetMetadataArgs) {
    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.append("Accept", HeaderValue::from_static("application/json"));

    let reqwest_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .default_headers(default_headers)
        .danger_accept_invalid_certs(config.accept_invalid_certs)
        .build()
        .unwrap();

    let discord_client = DiscordClient::new(
        reqwest_client.clone(),
        &config.discord_bot_token.sensitive_string(),
    );

    register_metadata(
        &config.discord_client_id.sensitive_string(),
        &discord_client,
    )
    .await
    .unwrap();
}

async fn register_metadata(application_id: &str, client: &DiscordClient) -> Result<()> {
    let metadata_spec = vec![
        ApplicationMetadataDefinition {
            key: "total_watches".into(),
            name: "Stream Count".into(),
            description: "Total watch count".into(),
            type_: 2,
        },
        ApplicationMetadataDefinition {
            key: "hours_watched".into(),
            name: "Hours Streamed".into(),
            description: "Hours spent streaming".into(),
            type_: 2,
        },
        ApplicationMetadataDefinition {
            key: "is_subscriber".into(),
            name: "✅".into(),
            description: "Access to Plex Server".into(),
            type_: 7,
        },
    ];

    let current_metadata: Vec<ApplicationMetadataDefinition> =
        client.application_metadata(application_id).await?;
    tracing::info!("Discord Metadata: {:#?}", current_metadata);
    if current_metadata != metadata_spec {
        tracing::info!("Registering Discord application metadata");
        client
            .register_application_metadata(application_id, metadata_spec)
            .await?;
    } else {
        tracing::info!("Discord application metadata is up to date")
    }

    Ok(())
}

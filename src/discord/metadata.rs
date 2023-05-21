use anyhow::Result;

use crate::config::ServerArgs;

use super::{
    client::DiscordClient,
    models::ApplicationMetadataDefinition,
};

pub async fn register_metadata(config: &ServerArgs, client: &DiscordClient) -> Result<()> {
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
            description: format!("Access to {}", config.application_name),
            type_: 7,
        },
    ];

    let current_metadata: Vec<ApplicationMetadataDefinition> =
        client.application_metadata().await?;
    tracing::info!("Discord Metadata: {:#?}", current_metadata);
    if current_metadata != metadata_spec {
        tracing::info!("Registering Discord application metadata");
        client.register_application_metadata(metadata_spec).await?;
    } else {
        tracing::info!("Discord application metadata is up to date")
    }

    Ok(())
}

use anyhow::Result;

use super::{
    client::DiscordClient,
    models::ApplicationMetadataDefinition,
};

pub async fn register_metadata(client: &DiscordClient) -> Result<()> {
    let metadata_spec = vec![
        ApplicationMetadataDefinition {
            key: "total_watches".into(),
            name: "Watch Count".into(),
            description: "Total watch count".into(),
            type_: 2,
        },
        ApplicationMetadataDefinition {
            key: "hours_watched".into(),
            name: "Hours Watched".into(),
            description: "Total hours spent watching".into(),
            type_: 2,
        },
        ApplicationMetadataDefinition {
            key: "is_subscriber".into(),
            name: "Is Subscribed?".into(),
            description: "Subscribed".into(),
            type_: 7,
        },
    ];

    let current_metadata: Vec<ApplicationMetadataDefinition> =
        client.application_metadata().await?;
    if current_metadata != metadata_spec {
        log::info!("Registering Discord application metadata");
        client.register_application_metadata(metadata_spec).await?;
    } else {
        log::info!("Discord application metadata is up to date")
    }

    Ok(())
}

use anyhow::Result;

use super::{client::DiscordClient, models::ApplicationMetadataDefinition};

pub async fn register_metadata(client: &DiscordClient) -> Result<()> {
    let metadata_spec = vec![
        ApplicationMetadataDefinition {
            key: "join_date".into(),
            name: "Joined".into(),
            description: "Date user joined".into(),
            type_: 6,
        },
        ApplicationMetadataDefinition {
            key: "hours_watched".into(),
            name: "Hours Watched".into(),
            description: "Hours watching".into(),
            type_: 2,
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

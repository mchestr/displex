use anyhow::Result;

use crate::{
    config::AppConfig,
    entities::{
        discord_user,
        plex_user,
    },
    services::{
        discord::models::{
            ApplicationMetadata,
            ApplicationMetadataUpdate,
        },
        tautulli::models::QueryDays,
        AppServices,
    },
};

pub async fn run(config: &AppConfig, services: &AppServices) -> Result<()> {
    let users = services
        .discord_users_service
        .list_users_for_refresh()
        .await
        .unwrap();
    tracing::info!("Refreshing {} users", users.len());
    for (discord_user, plex_user) in users {
        if plex_user.is_none() {
            anyhow::bail!("No plex user found! {:?}", discord_user);
        }
        let plex_user = plex_user.unwrap();
        match refresh_user_stats(config, services, &discord_user, &plex_user).await {
            Ok(_) => tracing::info!("successfully refreshed {}", discord_user.username),
            Err(err) => tracing::error!("failed to refresh {}: {err:?}", discord_user.username),
        };
    }
    Ok(())
}

async fn refresh_user_stats(
    config: &AppConfig,
    services: &AppServices,
    discord_user: &discord_user::Model,
    plex_user: &plex_user::Model,
) -> Result<()> {
    tracing::info!("refreshing stats for user {}", &discord_user.username);

    let discord_user_id = discord_user.id.clone();
    let discord_token = services
        .discord_tokens_service
        .latest_token(&discord_user_id)
        .await
        .unwrap();
    if discord_token.is_none() {
        anyhow::bail!(
            "unable to refresh {:?} as discord token does not exist.",
            discord_user.username
        );
    }
    let discord_token = discord_token.unwrap();

    let watch_stats = services
        .tautulli_service
        .get_user_watch_time_stats(&plex_user.id, Some(true), Some(QueryDays::Total))
        .await?;

    let latest_stat = watch_stats
        .first()
        .ok_or_else(|| anyhow::anyhow!("failed to fetch stats"))?;

    let metadata = ApplicationMetadataUpdate {
        platform_name: String::from(&config.application_name),
        metadata: ApplicationMetadata {
            watched_hours: latest_stat.total_time / 3600,
            is_subscribed: true,
        },
        ..Default::default()
    };
    tracing::info!("setting {} metadata: {:?}", discord_user.username, metadata);
    services
        .discord_service
        .link_application(
            config.discord.client_id,
            metadata,
            &discord_token.access_token,
        )
        .await?;
    Ok(())
}

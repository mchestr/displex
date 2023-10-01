use anyhow::Result;

use crate::{
    config::AppConfig,
    entities::{
        discord_user,
        plex_user,
    },
    overseerr::models,
    services::AppServices,
};

pub async fn run(services: &AppServices) -> Result<()> {
    let users = services
        .discord_users_service
        .list_subscribers()
        .await
        .unwrap();

    let overseerr_users = services.overseerr_service.get_users().await?;
    let mut overseerr_subscribers: Vec<i64> = vec![];
    tracing::info!("Processing request upgrades for {} users", users.len());
    for (discord_user, plex_user) in users {
        match plex_user {
            Some(plex_user) => {
                if !plex_user.is_subscriber {
                    tracing::warn!("Plex User {} is not a subscriber", plex_user.username);
                    continue;
                }
                match overseerr_users
                    .iter()
                    .find(|u| u.plex_id.to_string() == plex_user.id)
                {
                    Some(overseerr_user) => {
                        services
                            .overseerr_service
                            .set_request_tier(overseerr_user, &plex_user.id)
                            .await?;
                        overseerr_subscribers.push(overseerr_user.id);
                    }
                    None => tracing::warn!("No overseerr user found for {}", discord_user.username),
                }
            }
            None => tracing::warn!("no plex user found for {}", discord_user.username),
        }
    }

    for overseerr_user in overseerr_users {
        if overseerr_subscribers.contains(&overseerr_user.id) {
            continue;
        }
        tracing::info!(
            "Setting Overseerr user to default request settings: {}",
            overseerr_user.display_name
        );
        services
            .overseerr_service
            .set_default_request_settings(&overseerr_user)
            .await?;
    }
    Ok(())
}

pub async fn process_subscriber(
    _config: &AppConfig,
    services: &AppServices,
    overseerr_user: &models::User,
    discord_user: &discord_user::Model,
    plex_user: &plex_user::Model,
) -> Result<()> {
    tracing::info!("processing subscriber {}", &discord_user.username);
    services
        .overseerr_service
        .set_request_tier(overseerr_user, &plex_user.id)
        .await?;
    Ok(())
}

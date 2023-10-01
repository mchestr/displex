use std::time::Duration;

use anyhow::Result;
use oauth2::TokenResponse;

use crate::{
    config::AppConfig,
    discord::models::{
        ApplicationMetadata,
        ApplicationMetadataUpdate,
    },
    entities::{
        discord_token,
        discord_user,
        plex_user,
    },
    services::AppServices,
    tautulli::models::QueryDays,
};

pub async fn refresh_all_active_subscribers(
    config: &AppConfig,
    services: &AppServices,
) -> Result<()> {
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
        refresh_user_stats(config, services, &discord_user, &plex_user).await?;
        tracing::info!("successfully refreshed {}", discord_user.username);
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
        anyhow::bail!("no token found for user! {discord_user_id}")
    }
    let discord_token = discord_token.unwrap();

    let discord_token = match maybe_refresh_token(services, discord_user, discord_token).await {
        Ok(token) => token,
        Err(err) => {
            tracing::error!("Failed to refresh users token: {}", err);
            services
                .discord_users_service
                .deactivate(&discord_user_id)
                .await
                .unwrap();
            tracing::warn!("Deactivated user: {}", discord_user.username);
            return Err(err);
        }
    };

    let watch_stats = services
        .tautulli_service
        .get_user_watch_time_stats(&plex_user.id, Some(true), Some(QueryDays::Total))
        .await?;

    let latest_stat = watch_stats
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("failed to fetch stats"))?;

    services
        .discord_service
        .link_application(
            config.discord.client_id,
            ApplicationMetadataUpdate {
                platform_name: String::from(&config.application_name),
                metadata: ApplicationMetadata {
                    watched_hours: latest_stat.total_time / 3600,
                    is_subscribed: true,
                },
                ..Default::default()
            },
            &discord_token.access_token,
        )
        .await?;
    Ok(())
}

async fn maybe_refresh_token(
    services: &AppServices,
    discord_user: &discord_user::Model,
    discord_token: discord_token::Model,
) -> Result<discord_token::Model> {
    if discord_token.expires_at < chrono::Utc::now() + chrono::Duration::days(-1) {
        tracing::info!("refreshing token for user {}", &discord_user.username);
        let new_token = services
            .discord_service
            .refresh_token(&discord_token.refresh_token)
            .await?;
        let expires_at = chrono::Utc::now()
            + chrono::Duration::seconds(
                new_token
                    .expires_in()
                    .unwrap_or_else(|| {
                        tracing::error!(
                        "failed to figure out when token will expire, defaulting to 3 days for {}",
                        discord_user.username
                    );
                        Duration::from_secs(3600 * 24 * 3)
                    })
                    .as_secs() as i64,
            );

        let discord_user = discord_user.clone();
        services
            .discord_tokens_service
            .create(
                new_token.access_token().secret(),
                new_token
                    .refresh_token()
                    .expect("no refresh token returned!")
                    .secret(),
                &expires_at,
                &discord_token.scopes,
                &discord_user.id,
            )
            .await
            .unwrap();
        Ok(discord_token::Model {
            access_token: new_token.access_token().secret().to_owned(),
            ..Default::default()
        })
    } else {
        Ok(discord_token)
    }
}

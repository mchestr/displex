use std::time::Duration;

use anyhow::{
    anyhow,
    Result,
};
use chrono::Utc;
use oauth2::TokenResponse;

use crate::{
    config::AppConfig,
    entities::discord_token,
    services::AppServices,
};

pub async fn run(config: &AppConfig, services: &AppServices) -> Result<()> {
    let tokens = services
        .discord_tokens_service
        .list(None, None)
        .await
        .map_err(|err| anyhow!(err.message))?;

    let now = Utc::now();
    for token in tokens {
        if token.expires_at <= now {
            tracing::info!("Removing token for user: {:?}", token.discord_user_id);
            services
                .discord_tokens_service
                .delete(&token.access_token)
                .await
                .map_err(|err| anyhow!(err.message))?;
        } else if token.expires_at < now + config.token_maintenance.refresh_days_to_expire {
            tracing::info!(
                "User {:?} token expires at {:?}, refreshing token...",
                token.discord_user_id,
                token.expires_at
            );
            match refresh_token(services, &token).await {
                Ok(_) => {
                    tracing::info!("Success");
                    services
                        .discord_tokens_service
                        .delete(&token.access_token)
                        .await
                        .map_err(|err| anyhow!(err.message))?;
                }
                Err(err) => tracing::error!("error: {:?}", err),
            };
        }
    }
    Ok(())
}

async fn refresh_token(services: &AppServices, discord_token: &discord_token::Model) -> Result<()> {
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
                        "failed to figure out when token will expire, defaulting to 7 days for {}",
                        discord_token.discord_user_id
                    );
                    Duration::from_secs(3600 * 24 * 7)
                })
                .as_secs() as i64,
        );

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
            &discord_token.discord_user_id,
        )
        .await
        .unwrap();
    Ok(())
}
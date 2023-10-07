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
        .list(None, None, Some(discord_token::TokenStatus::Active))
        .await
        .map_err(|err| anyhow!(err.message))?;

    let now = Utc::now();
    for token in tokens {
        if token.expires_at <= now {
            tracing::info!("token for user expired: {:?}", token.discord_user_id);
            services
                .discord_tokens_service
                .set_status(&token.access_token, discord_token::TokenStatus::Expired)
                .await
                .map_err(|err| anyhow!(err.message))?;
        } else if token.expires_at < now + config.token_maintenance.refresh_days_to_expire {
            tracing::info!(
                "user {:?} token expires at {:?}, refreshing token...",
                token.discord_user_id,
                token.expires_at
            );
            match refresh_token(services, &token).await {
                Ok(_) => {
                    tracing::info!("Success");
                    services
                        .discord_tokens_service
                        .set_status(&token.access_token, discord_token::TokenStatus::Renewed)
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
    let new_token = match services
        .discord_service
        .refresh_token(&discord_token.refresh_token)
        .await
    {
        Ok(token) => token,
        Err(err) => {
            tracing::error!("failed to refresh token {err:?}... revoking.");
            services
                .discord_service
                .revoke_token(&discord_token.refresh_token)
                .await?;
            services
                .discord_tokens_service
                .set_status(
                    &discord_token.access_token,
                    discord_token::TokenStatus::Revoked,
                )
                .await
                .map_err(|err| anyhow!(err.message))?;
            return Ok(());
        }
    };
    tracing::info!("new token: {:?}", new_token);
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

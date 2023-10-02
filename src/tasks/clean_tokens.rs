use anyhow::{
    anyhow,
    Result,
};
use chrono::{
    Duration,
    DurationRound,
};

use crate::services::AppServices;

pub async fn run(services: &AppServices) -> Result<()> {
    let yesterday = chrono::Utc::now().duration_round(Duration::days(0))?;
    let tokens = services
        .discord_tokens_service
        .list(None, Some(yesterday))
        .await
        .map_err(|err| anyhow!(err.message))?;

    for token in tokens {
        tracing::info!(
            "cleaning up expired token for user {}, expired {}",
            token.discord_user_id,
            token.expires_at
        );
        services
            .discord_tokens_service
            .delete(&token.access_token)
            .await
            .map_err(|err| anyhow!(err.message))?;
    }
    Ok(())
}

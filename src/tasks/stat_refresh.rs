use std::{
    io::{
        Error,
        ErrorKind,
    },
    time::Duration,
};

use crate::{
    config::RefreshArgs,
    db::{
        discord::{
            get_latest_token,
            insert_token,
            DiscordToken,
            DiscordUser,
            NewDiscordToken,
        },
        initialize_db_pool,
        list_users,
        plex::PlexUser,
    },
    discord::{
        client::DiscordClient,
        models::{
            ApplicationMetadata,
            ApplicationMetadataUpdate,
        },
    },
    tautulli::{
        client::TautulliClient,
        models::QueryDays,
    },
};
use anyhow::Result;
use oauth2::TokenResponse;
use reqwest::header::HeaderValue;
use sqlx::{
    Pool,
    Postgres,
};
use tracing::instrument;

pub async fn run(config: RefreshArgs) -> std::io::Result<()> {
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
        &reqwest_client,
        &config.discord.discord_client_id.sensitive_string(),
        &config.discord.discord_client_secret.sensitive_string(),
        &format!("https://{}/discord/callback", &config.hostname),
        &config.discord.discord_bot_token.sensitive_string(),
    );

    let tautlli_client = TautulliClient::new(
        &reqwest_client.clone(),
        &config.tautulli.tautulli_url,
        &config.tautulli.tautulli_api_key.sensitive_string(),
    );

    let pool = initialize_db_pool(&config.database.database_url.sensitive_string())
        .await
        .unwrap();

    let users = list_users(&pool).await.unwrap();
    tracing::info!("Refreshing {} users", users.len());
    for (discord_user, plex_user) in users {
        match refresh_user_stats(
            &config,
            &pool,
            &discord_client,
            &tautlli_client,
            &discord_user,
            &plex_user,
        )
        .await
        {
            Ok(_) => tracing::info!("Successfully refreshed {}", &discord_user.username),
            Err(err) => {
                tracing::error!("Failed to refresh user {}: {}", &discord_user.username, err)
            }
        }
    }
    Ok(())
}

#[instrument(skip(config, pool, discord_client, tautulli_client))]
async fn refresh_user_stats(
    config: &RefreshArgs,
    pool: &Pool<Postgres>,
    discord_client: &DiscordClient,
    tautulli_client: &TautulliClient,
    discord_user: &DiscordUser,
    plex_user: &PlexUser,
) -> Result<()> {
    tracing::info!("refreshing stats for user {}", &discord_user.username);

    let discord_user_id = discord_user.id.clone();
    let discord_token = get_latest_token(pool, &discord_user_id).await.unwrap();

    let discord_token =
        maybe_refresh_token(pool, discord_client, discord_user, discord_token).await?;

    let watch_stats = tautulli_client
        .get_user_watch_time_stats(plex_user.id, Some(true), Some(QueryDays::Total))
        .await?;

    let latest_stat = watch_stats
        .get(0)
        .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to get latest stats"))?;
    discord_client
        .link_application(
            &discord_token.access_token,
            ApplicationMetadataUpdate {
                platform_name: String::from(&config.application_name),
                metadata: ApplicationMetadata {
                    total_watches: latest_stat.total_plays,
                    hours_watched: latest_stat.total_time / 3600,
                    is_subscriber: true,
                },
                ..Default::default()
            },
        )
        .await?;
    Ok(())
}

async fn maybe_refresh_token(
    conn: &Pool<Postgres>,
    discord_client: &DiscordClient,
    discord_user: &DiscordUser,
    discord_token: DiscordToken,
) -> Result<DiscordToken> {
    if discord_token.expires_at < chrono::Utc::now() + chrono::Duration::days(-1) {
        tracing::info!("refreshing token for user {}", &discord_user.username);
        let new_token = discord_client
            .refresh_token(&discord_token.refresh_token)
            .await?;

        let discord_user = discord_user.clone();
        let inserted_token = insert_token(
            conn,
            NewDiscordToken {
                access_token: new_token.access_token().secret().into(),
                refresh_token: new_token
                    .refresh_token()
                    .ok_or_else(|| Error::new(ErrorKind::Other, "No refresh token returned!"))?
                    .secret()
                    .into(),
                scopes: discord_token.scopes,
                expires_at: chrono::Utc::now()
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
                    ),
                discord_user_id: discord_user.id.clone(),
            },
        ).await.unwrap();
        Ok(inserted_token)
    } else {
        Ok(discord_token)
    }
}

use std::time::Duration;

use anyhow::Result;
use oauth2::TokenResponse;
use sqlx::{
    PgConnection,
    PgPool,
};
use tokio::{
    select,
    time,
};

use crate::{
    db::{
        discord::{
            get_latest_token,
            insert_token,
            DiscordToken,
            DiscordUser,
            NewDiscordToken,
        },
        list_users,
        plex::PlexUser,
    },
    discord::{
        client::{
            DiscordClient,
            DiscordOAuth2Client,
        },
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

pub struct UserMetadataRefreshArgs {
    pub application_name: String,
    pub client_id: String,
    pub update_interval: Duration,
    pub pool: PgPool,
    pub discord_client: DiscordClient,
    pub discord_oauth_client: DiscordOAuth2Client,
    pub tautulli_client: TautulliClient,
}

pub async fn setup(kill: tokio::sync::broadcast::Receiver<()>, args: UserMetadataRefreshArgs) {
    tracing::info!(
        "refreshing user metadata every {}s",
        args.update_interval.as_secs()
    );
    tokio::spawn(periodic_refresh(kill, args));
}

async fn periodic_refresh(
    mut kill: tokio::sync::broadcast::Receiver<()>,
    args: UserMetadataRefreshArgs,
) {
    let mut interval = time::interval(args.update_interval);
    loop {
        select! {
            _ = interval.tick() => {
                refresh_all_active_subscribers(&args).await;
            },
            _ = kill.recv() => {
                tracing::info!("shutting down periodic job...");
                return;
            },
        }
    }
}

async fn refresh_all_active_subscribers(args: &UserMetadataRefreshArgs) {
    let mut conn = args.pool.acquire().await.unwrap();
    let users = list_users(&mut conn).await.unwrap();
    tracing::info!("Refreshing {} users", users.len());
    for (discord_user, plex_user) in users {
        match refresh_user_stats(&mut conn, args, &discord_user, &plex_user).await {
            Ok(_) => tracing::info!("Successfully refreshed {}", &discord_user.username),
            Err(err) => {
                tracing::error!("Failed to refresh user {}: {}", &discord_user.username, err)
            }
        }
    }
}

async fn refresh_user_stats(
    conn: &mut PgConnection,
    args: &UserMetadataRefreshArgs,
    discord_user: &DiscordUser,
    plex_user: &PlexUser,
) -> Result<()> {
    tracing::info!("refreshing stats for user {}", &discord_user.username);

    let discord_user_id = discord_user.id.clone();
    let discord_token = get_latest_token(conn, &discord_user_id).await.unwrap();

    let discord_token = maybe_refresh_token(
        conn,
        &args.discord_oauth_client,
        discord_user,
        discord_token,
    )
    .await?;

    let watch_stats = args
        .tautulli_client
        .get_user_watch_time_stats(plex_user.id, Some(true), Some(QueryDays::Total))
        .await?;

    let latest_stat = watch_stats
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("failed to fetch stats"))?;

    args.discord_client
        .link_application(
            &args.client_id,
            ApplicationMetadataUpdate {
                platform_name: String::from(&args.application_name),
                metadata: ApplicationMetadata {
                    total_watches: latest_stat.total_plays,
                    hours_watched: latest_stat.total_time / 3600,
                    is_subscriber: true,
                },
                ..Default::default()
            },
            &discord_token.access_token,
        )
        .await?;
    Ok(())
}

async fn maybe_refresh_token(
    conn: &mut PgConnection,
    discord_oauth_client: &DiscordOAuth2Client,
    discord_user: &DiscordUser,
    discord_token: DiscordToken,
) -> Result<DiscordToken> {
    if discord_token.expires_at < chrono::Utc::now() + chrono::Duration::days(-1) {
        tracing::info!("refreshing token for user {}", &discord_user.username);
        let new_token = discord_oauth_client
            .refresh_token(&discord_token.refresh_token)
            .await?;

        let discord_user = discord_user.clone();
        let inserted_token = insert_token(
            conn,
            NewDiscordToken {
                access_token: new_token.access_token().secret().into(),
                refresh_token: new_token
                    .refresh_token()
                    .ok_or_else(|| anyhow::anyhow!("No refresh token returned!"))?
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

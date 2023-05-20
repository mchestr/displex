use std::time::Duration;

use crate::{
    config::RefreshArgs,
    db::{
        discord::{
            get_latest_token,
            insert_token,
            NewDiscordToken,
        },
        initialize_db_pool,
        list_users,
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
use oauth2::TokenResponse;
use reqwest::header::HeaderValue;

async fn main(config: RefreshArgs) {
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
        &config.discord.discord_server_id,
        &config.discord.discord_channel_id,
    );

    let tautlli_client = TautulliClient::new(
        &reqwest_client.clone(),
        &config.tautulli.tautulli_url,
        &config.tautulli.tautulli_api_key.sensitive_string(),
    );

    let pool = initialize_db_pool(&config.database.database_url.sensitive_string());
    let mut conn = pool.get().unwrap();

    let users = list_users(&mut conn).unwrap();
    let expire_window = chrono::Utc::now() + chrono::Duration::days(-1);

    log::info!("Refreshing {} users", users.len());
    for (discord_user, plex_user) in users {
        log::info!("refreshing stats for user {}", &discord_user.username);
        let mut discord_token = get_latest_token(&mut conn, &discord_user.id).unwrap();

        if discord_token.expires_at < expire_window {
            log::info!("refreshing token for user {}", &discord_user.username);
            let new_token = discord_client
                .refresh_token(&discord_token.refresh_token)
                .await
                .unwrap();
            let new_token = insert_token(
                &mut conn,
                NewDiscordToken {
                    access_token: new_token.access_token().secret().into(),
                    refresh_token: new_token
                        .refresh_token()
                        .expect("expecting refresh token")
                        .secret()
                        .into(),
                    scopes: discord_token.scopes,
                    expires_at: chrono::Utc::now()
                        + chrono::Duration::seconds(
                            new_token
                                .expires_in()
                                .unwrap_or(Duration::from_secs(1800))
                                .as_secs() as i64,
                        ),
                    discord_user_id: discord_user.id,
                },
            )
            .unwrap();
            discord_token = new_token;
        }
        let watch_stats = tautlli_client
            .get_user_watch_time_stats(plex_user.id, Some(true), Some(QueryDays::Total))
            .await
            .unwrap();

        let latest_stat = watch_stats.get(0).unwrap();

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
                },
            )
            .await
            .unwrap();
    }
}

pub fn run(config: RefreshArgs) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { main(config.clone()).await });
}

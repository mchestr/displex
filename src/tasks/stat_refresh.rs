use std::time::Duration;

use displex::{
    config::Config,
    db::{
        discord::{get_latest_token, insert_token, NewDiscordToken},
        initialize_db_pool, list_users,
    },
    discord::{
        client::DiscordClient,
        models::{ApplicationMetadata, ApplicationMetadataUpdate},
    },
    tautulli::client::TautulliClient,
};
use dotenv::dotenv;
use oauth2::TokenResponse;
use reqwest::header::HeaderValue;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::init();
    log::info!("Loaded config: {}", config);

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
        &config.discord_client_id,
        &config.discord_client_secret,
        &format!("https://{}/discord/callback", &config.hostname),
        &config.discord_bot_token,
        &config.discord_server_id,
        &config.discord_channel_id,
    );

    let tautlli_client = TautulliClient::new(
        &reqwest_client.clone(),
        &config.tautulli_url,
        &config.tautulli_api_key,
    );

    let pool = initialize_db_pool(&config.database_url);
    let mut conn = pool.get().unwrap();

    let users = list_users(&mut conn).unwrap();
    let expire_window = chrono::Utc::now() + chrono::Duration::days(-1);

    log::info!("Refreshing {} users", users.len());
    for (discord_user, plex_user) in users {
        log::info!("refreshing stats for user {:?}", &discord_user.id);
        let mut discord_token = get_latest_token(&mut conn, &discord_user.id).unwrap();

        if discord_token.expires_at < expire_window {
            log::info!("refreshing token for user {:?}", &discord_user.id);
            let new_token = discord_client
                .refresh_token(&discord_token.access_token)
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
            .get_user_watch_time_stats(plex_user.id, Some(false), Some("0"))
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
                    },
                },
            )
            .await
            .unwrap();
    }
}

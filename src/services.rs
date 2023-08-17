use std::time::Duration;

use axum::http::HeaderValue;
use sea_orm::DatabaseConnection;
use serenity::http::HttpBuilder;

use crate::{
    bot,
    config::AppConfig,
    discord::DiscordService,
    discord_token::resolver::DiscordTokensService,
    discord_user::resolver::DiscordUsersService,
    overseerr::OverseerrService,
    plex::PlexService,
    plex_token::resolver::PlexTokensService,
    plex_user::resolver::PlexUsersService,
    tautulli::TautulliService,
};

/// All the services that are used by the app
#[derive(Clone)]
pub struct AppServices {
    pub discord_users_service: DiscordUsersService,
    pub discord_tokens_service: DiscordTokensService,
    pub plex_users_service: PlexUsersService,
    pub plex_tokens_service: PlexTokensService,
    pub tautulli_service: TautulliService,
    pub discord_service: DiscordService,
    pub plex_service: PlexService,
    pub overseerr_service: OverseerrService,
}

pub async fn create_app_services(
    db: DatabaseConnection,
    config: &AppConfig,
) -> (serenity::Client, AppServices) {
    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.append("Accept", HeaderValue::from_static("application/json"));

    let reqwest_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .default_headers(default_headers)
        .danger_accept_invalid_certs(config.debug.accept_invalid_certs)
        .build()
        .unwrap();

    let discord_tokens_service = DiscordTokensService::new(&db);
    let plex_users_service = PlexUsersService::new(&db);
    let plex_tokens_service = PlexTokensService::new(&db);
    let discord_users_service = DiscordUsersService::new(
        &db,
        &discord_tokens_service,
        &plex_tokens_service,
        &plex_users_service,
    );
    let tautulli_service = TautulliService::new(
        &reqwest_client,
        &config.tautulli.url,
        &config.tautulli.api_key,
    );

    let http_client = HttpBuilder::new(&config.discord_bot.token)
        .client(reqwest_client.clone())
        .build();
    let serenity_client = bot::discord::init(config.clone(), http_client)
        .await
        .unwrap();

    let discord_service = DiscordService::new(
        &reqwest_client,
        &serenity_client.cache_and_http.http.clone(),
        &config.discord_bot.token,
        config.discord.client_id,
        &config.discord.client_secret,
        &format!("https://{}/auth/discord/callback", &config.http.hostname),
    );
    let plex_service = PlexService::new(
        &reqwest_client,
        &config.application_name,
        &format!("https://{}/auth/plex/callback", &config.http.hostname),
    );
    let overseerr_service = OverseerrService::new(
        &reqwest_client,
        &config.overseerr.url,
        &config.overseerr.api_key,
    );
    (
        serenity_client,
        AppServices {
            discord_users_service,
            discord_tokens_service,
            plex_users_service,
            plex_tokens_service,
            tautulli_service,
            discord_service,
            plex_service,
            overseerr_service,
        },
    )
}

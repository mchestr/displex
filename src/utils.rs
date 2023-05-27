use std::{time::Duration, sync::Arc};

use anyhow::Result;
use axum::http::HeaderValue;

use sqlx::{
    Pool,
    Postgres,
};

use crate::{
    config::DisplexConfig,
    db,
    discord::client::{
        DiscordClient,
        DiscordOAuth2Client,
    },
    tautulli::client::TautulliClient, bot, plex::client::PlexClient,
};

pub struct DisplexClients {
    pub reqwest_client: reqwest::Client,
    pub tautulli_client: TautulliClient,
    pub pool: Pool<Postgres>,
    pub discord_oauth2_client: DiscordOAuth2Client,
    pub discord_client: DiscordClient,
    pub serenity_client: Arc<serenity::http::Http>,
    pub plex_client: PlexClient,
}

pub async fn initialize_clients(config: &DisplexConfig) -> Result<(serenity::Client, DisplexClients)> {
    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.append("Accept", HeaderValue::from_static("application/json"));

    let reqwest_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .default_headers(default_headers)
        .build()
        .unwrap();

    let tautulli_client = TautulliClient::new(
        &reqwest_client,
        &config.tautulli.url,
        &config.tautulli.api_key,
    );

    let pool = db::initialize_db_pool(&config.database.url).await.unwrap();

    let discord_client = DiscordClient::new(reqwest_client.clone(), &config.discord_bot.token);
    let discord_oauth2_client = DiscordOAuth2Client::new(
        reqwest_client.clone(),
        config.discord.client_id,
        &config.discord.client_secret,
        Some(&format!(
            "https://{}/discord/callback",
            &config.http.hostname
        )),
    );

    let plex_client = PlexClient::new(
        &reqwest_client,
        &config.application_name,
        &format!("https://{}/plex/callback", &config.http.hostname),
    );

    let serenity_client = bot::discord::init(config.clone()).await?;
    let http_client = serenity_client.cache_and_http.http.clone();

    Ok((serenity_client, DisplexClients {
        reqwest_client,
        tautulli_client,
        pool,
        discord_oauth2_client,
        discord_client,
        serenity_client: http_client,
        plex_client,
    }))
}

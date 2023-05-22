use std::time::Duration;

use axum::{
    extract::MatchedPath,
    http::{
        HeaderValue,
        Request,
    },
    Router,
};
use axum_sessions::{
    async_session::CookieStore,
    SessionLayer,
};
use sqlx::{
    Pool,
    Postgres,
};
use tokio::sync::broadcast::Receiver;
use tower_http::trace::TraceLayer;
use tracing::info_span;

use crate::{
    config::ServerArgs,
    db::{self,},
    discord::client::{
        DiscordClient,
        DiscordOAuth2Client,
    },
    plex::client::PlexClient,
    tautulli::client::TautulliClient,
};

mod errors;
mod routes;

#[derive(Clone)]
pub struct DisplexState {
    pub config: ServerArgs,
    pub discord_client: DiscordClient,
    pub discord_oauth_client: DiscordOAuth2Client,
    pub plex_client: PlexClient,
    pub tautulli_client: TautulliClient,
    pub db: Pool<Postgres>,
}

pub async fn run(mut kill: Receiver<()>, config: ServerArgs) {
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
        reqwest_client.clone(),
        &config.discord.discord_bot_token.sensitive_string(),
    );

    let discord_oauth_client = DiscordOAuth2Client::new(
        reqwest_client.clone(),
        &config.discord.discord_client_id.sensitive_string(),
        &config.discord.discord_client_secret.sensitive_string(),
        Some(&format!("https://{}/discord/callback", &config.hostname)),
    );

    let plex_client = PlexClient::new(
        &reqwest_client,
        &config.application_name,
        &format!("https://{}/plex/callback", &config.hostname),
    );

    let tautulli_client = TautulliClient::new(
        &reqwest_client,
        &config.tautulli.tautulli_url,
        &config.tautulli.tautulli_api_key.sensitive_string(),
    );

    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(MatchedPath::as_str);

        let request_id = uuid::Uuid::new_v4().to_string();
        info_span!(
            "req",
            id = request_id,
            method = ?request.method(),
            matched_path,
            some_other_field = tracing::field::Empty,
        )
    });

    let store = CookieStore::new();
    let secret = config.session.session_secret_key.sensitive_string();
    let session_layer = SessionLayer::new(store, secret.as_bytes())
        .with_secure(true)
        .with_same_site_policy(axum_sessions::SameSite::Lax)
        .with_cookie_domain(&config.hostname);

    let db = db::initialize_db_pool(&config.database.database_url.sensitive_string())
        .await
        .unwrap();

    db::run_migrations(&db).await.unwrap();

    let addr = format!("{}:{}", &config.host, &config.port);

    let app = Router::new()
        .merge(routes::configure())
        .with_state(DisplexState {
            config,
            discord_client,
            discord_oauth_client,
            plex_client,
            tautulli_client,
            db,
        })
        .layer(session_layer)
        .layer(trace_layer);

    tracing::info!("starting server on {}", &addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            tokio::select! {
                _ = kill.recv() => tracing::info!("shutting down http server..."),
            }
        })
        .await
        .unwrap();
}

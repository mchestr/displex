

use axum::{
    extract::MatchedPath,
    http::{
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
    config::DisplexConfig,
    db::{self,},
    discord::client::{
        DiscordClient,
        DiscordOAuth2Client,
    },
    plex::client::PlexClient,
    tautulli::client::TautulliClient, utils::DisplexClients,
};

mod errors;
mod routes;

pub const DISCORD_CODE: &str = "code";
pub const DISCORD_STATE: &str = "state";

#[derive(Clone)]
pub struct DisplexState {
    pub config: DisplexConfig,
    pub discord_client: DiscordClient,
    pub discord_oauth_client: DiscordOAuth2Client,
    pub plex_client: PlexClient,
    pub tautulli_client: TautulliClient,
    pub db: Pool<Postgres>,
}

pub async fn run(mut kill: Receiver<()>, config: DisplexConfig, clients: &DisplexClients) {
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
    let secret = &config.session.secret_key;
    let session_layer = SessionLayer::new(store, secret.as_bytes())
        .with_secure(true)
        .with_same_site_policy(axum_sessions::SameSite::Lax)
        .with_cookie_domain(&config.http.hostname);

    let db = db::initialize_db_pool(&config.database.url).await.unwrap();

    db::run_migrations(&db).await.unwrap();

    let addr = format!("{}:{}", &config.http.host, &config.http.port);
    let app = Router::new()
        .merge(routes::configure())
        .with_state(DisplexState {
            config,
            discord_client: clients.discord_client.clone(),
            discord_oauth_client: clients.discord_oauth2_client.clone(),
            plex_client: clients.plex_client.clone(),
            tautulli_client: clients.tautulli_client.clone(),
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

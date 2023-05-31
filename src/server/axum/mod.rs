use axum::{
    Extension,
    Router,
};
use axum_sessions::{
    async_session::CookieStore,
    SessionLayer,
};
use reqwest::{
    header,
    Method,
};
use tokio::sync::broadcast::Receiver;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};

use crate::{
    config::AppConfig,
    graphql::GraphqlSchema,
    services::AppServices,
};

mod errors;
mod routes;

pub const DISCORD_CODE: &str = "code";
pub const DISCORD_STATE: &str = "state";

#[derive(Clone)]
pub struct DisplexState {
    pub config: AppConfig,
    pub services: AppServices,
}

pub async fn run(
    mut kill: Receiver<()>,
    config: AppConfig,
    services: &AppServices,
    schema: &GraphqlSchema,
) {
    let store = CookieStore::new();
    let secret = &config.session.secret_key;
    let session_layer = SessionLayer::new(store, secret.as_bytes())
        .with_secure(true)
        .with_same_site_policy(axum_sessions::SameSite::Lax)
        .with_cookie_domain(&config.http.hostname);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::ACCEPT, header::CONTENT_TYPE])
        .allow_origin(
            config
                .web
                .cors_origins
                .iter()
                .map(|f| f.parse().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_credentials(true);

    let addr = format!("{}:{}", &config.http.host, &config.http.port);
    let app = Router::new()
        .merge(routes::configure())
        .with_state(DisplexState {
            config,
            services: services.clone(),
        })
        .layer(session_layer)
        .layer(Extension(schema.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new())
        .layer(CookieManagerLayer::new())
        .layer(cors);

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

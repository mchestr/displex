use axum::{
    Extension,
    Router,
};

use prometheus_client::registry::Registry;
use tokio::{
    net::TcpListener,
    sync::broadcast::Receiver,
};
use tower_cookies::CookieManagerLayer;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};

use crate::{
    config::AppConfig,
    graphql::GraphqlSchema,
    metrics::{
        new_registry,
        Metrics,
    },
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
    pub metrics: Metrics,
    pub registry: std::sync::Arc<Registry>,
}

pub async fn run(
    _kill: Receiver<()>,
    config: AppConfig,
    services: &AppServices,
    schema: &GraphqlSchema,
) {
    let cors = CorsLayer::new()
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_headers([http::header::ACCEPT, http::header::CONTENT_TYPE])
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
    let metrics = Metrics::new();
    let registry = std::sync::Arc::new(new_registry(&metrics));
    let app = Router::new()
        .merge(routes::configure(&config))
        .with_state(DisplexState {
            config,
            services: services.clone(),
            metrics,
            registry,
        })
        .layer(CookieManagerLayer::new())
        .layer(Extension(schema.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new())
        .layer(CookieManagerLayer::new())
        .layer(cors);

    tracing::info!("starting server on {}", &addr);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

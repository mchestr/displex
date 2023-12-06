use axum::{
    extract::State,
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Router,
};

use prometheus_client::encoding::text::encode;

use crate::config::AppConfig;

use super::DisplexState;

mod discord;
mod plex;

async fn metrics(State(state): State<DisplexState>) -> impl IntoResponse {
    let mut body = String::new();
    encode(&mut body, &state.registry).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::CONTENT_TYPE,
        "application/openmetrics-text; version=1.0.0; charset=utf-8"
            .parse()
            .unwrap(),
    );
    (headers, body)
}

pub fn configure(_config: &AppConfig) -> Router<DisplexState> {
    Router::new()
        .merge(discord::routes())
        .merge(plex::routes())
        .route("/metrics", get(metrics))
}

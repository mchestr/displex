use axum::{
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_sessions::extractors::{
    ReadableSession,
    WritableSession,
};

use crate::server::axum::DISCORD_STATE;

use super::DisplexState;

mod discord;
mod plex;

async fn display_handler(session: ReadableSession) -> impl IntoResponse {
    let mut count: String = "test".into();
    count = session.get(DISCORD_STATE).unwrap_or(count);
    format!("Count is: {count}; visit /inc to increment and /reset to reset")
}

async fn increment_handler(mut session: WritableSession) -> impl IntoResponse {
    session.insert("count", String::from("testss")).unwrap();
}

async fn reset_handler(mut session: WritableSession) -> impl IntoResponse {
    session.destroy();
    "Count reset"
}

pub fn configure() -> Router<DisplexState> {
    Router::new()
        .route("/", get(display_handler))
        .route("/inc", get(increment_handler))
        .route("/reset", get(reset_handler))
        .merge(discord::routes())
        .merge(plex::routes())
}

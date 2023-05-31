use anyhow::anyhow;
use axum::{
    extract::{
        Query,
        State,
    },
    response::{
        IntoResponse,
        Redirect,
    },
    routing::get,
    Router,
};
use axum_sessions::extractors::WritableSession;
use serde::Deserialize;

use crate::{
    errors::DisplexError,
    server::axum::{
        DisplexState,
        DISCORD_CODE,
        DISCORD_STATE,
    },
};

async fn linked_role(
    mut session: WritableSession,
    State(state): State<DisplexState>,
) -> Result<impl IntoResponse, DisplexError> {
    let (url, state) = state.services.discord_service.authorize_url();
    session.insert(DISCORD_STATE, state.secret())?;

    Ok(Redirect::to(url.as_str()))
}

#[derive(Deserialize)]
struct CallbackQueryParams {
    pub code: String,
    pub state: String,
}

async fn callback(
    mut session: WritableSession,
    State(state): State<DisplexState>,
    query_string: Query<CallbackQueryParams>,
) -> Result<impl IntoResponse, DisplexError> {
    let session_state = session
        .get::<String>(DISCORD_STATE)
        .ok_or_else(|| anyhow!("session state is invalid"))?;
    verify_state(&session_state, &query_string.state)?;

    session.insert(DISCORD_CODE, &query_string.code)?;

    let pin = state.services.plex_service.get_pin().await?;
    let url = state
        .services
        .plex_service
        .generate_auth_url(pin.id, &pin.code)
        .await?;

    Ok(Redirect::to(url.as_str()))
}

#[tracing::instrument]
fn verify_state(session_state: &str, query_string_state: &str) -> Result<(), anyhow::Error> {
    if session_state != query_string_state {
        tracing::info!("session state does not match query parameters");
        anyhow::bail!("invalid state")
    }
    Ok(())
}

pub fn routes() -> Router<DisplexState> {
    Router::new()
        .route("/auth/discord/linked-role", get(linked_role))
        .route("/auth/discord/callback", get(callback))
}

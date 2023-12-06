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
use serde::Deserialize;
use tower_cookies::{
    Cookie,
    Cookies,
};

use crate::{
    errors::DisplexError,
    server::axum::{
        DisplexState,
        DISCORD_CODE,
        DISCORD_STATE,
    },
};

async fn linked_role(
    cookies: Cookies,
    State(state): State<DisplexState>,
) -> Result<impl IntoResponse, DisplexError> {
    let (url, state) = state.services.discord_service.authorize_url();

    let state = String::from(state.secret());
    cookies.add(Cookie::new(DISCORD_STATE, state));

    Ok(Redirect::to(url.as_str()))
}

#[derive(Deserialize)]
struct CallbackQueryParams {
    pub code: String,
    pub state: String,
}

async fn callback(
    cookies: Cookies,
    State(state): State<DisplexState>,
    query_string: Query<CallbackQueryParams>,
) -> Result<impl IntoResponse, DisplexError> {
    let session_state = cookies
        .get(DISCORD_STATE)
        .ok_or_else(|| anyhow!("session state is invalid"))?;
    verify_state(session_state.value(), &query_string.state)?;

    let code = String::from(&query_string.code);
    cookies.add(Cookie::new(DISCORD_CODE, code));

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

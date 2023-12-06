use std::time::Duration;

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
use cookie::{
    time::OffsetDateTime,
    Key,
    SameSite,
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
    let (url, persist_state) = state.services.discord_service.authorize_url();

    let persist_state = String::from(persist_state.secret());

    let key = Key::from(state.config.session.secret_key.as_bytes());
    let signed = cookies.signed(&key);
    let cookie = Cookie::build((DISCORD_STATE, persist_state))
        .same_site(SameSite::Lax)
        .http_only(true)
        .secure(true)
        .path("/")
        .expires(OffsetDateTime::now_utc() + Duration::from_secs(300))
        .build();
    signed.add(cookie);

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
    let key = Key::from(state.config.session.secret_key.as_bytes());
    let signed = cookies.signed(&key);
    let session_state = signed
        .get(DISCORD_STATE)
        .ok_or_else(|| anyhow!("session state is invalid"))?;
    verify_state(session_state.value(), &query_string.state)?;

    let code = String::from(&query_string.code);
    let key = Key::from(state.config.session.secret_key.as_bytes());
    let signed = cookies.signed(&key);
    let cookie = Cookie::build((DISCORD_CODE, code))
        .same_site(SameSite::Lax)
        .http_only(true)
        .secure(true)
        .path("/")
        .expires(OffsetDateTime::now_utc() + Duration::from_secs(300))
        .build();
    signed.add(cookie);

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

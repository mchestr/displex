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
use axum_sessions::extractors::ReadableSession;
use oauth2::TokenResponse;
use serde::Deserialize;

use crate::{
    discord::models::{
        ApplicationMetadata,
        ApplicationMetadataUpdate,
    },
    discord_token::resolver::{
        CreateDiscordTokenErrorVariant,
        CreateDiscordTokenResult,
    },
    discord_user::resolver::{
        CreateDiscordUserErrorVariant,
        CreateDiscordUserResult,
    },
    errors::DisplexError,
    plex_token::resolver::{
        CreatePlexTokenErrorVariant,
        CreatePlexTokenResult,
    },
    plex_user::resolver::{
        CreatePlexUserErrorVariant,
        CreatePlexUserResult,
    },
    server::axum::{
        DisplexState,
        DISCORD_CODE,
    },
    tautulli::models::QueryDays,
};

#[derive(Deserialize)]
struct CallbackQueryParams {
    pub id: u64,
    pub code: String,
}

async fn callback(
    session: ReadableSession,
    State(state): State<DisplexState>,
    query_string: Query<CallbackQueryParams>,
) -> Result<impl IntoResponse, DisplexError> {
    let plex_svc = state.services.plex_service;
    let tautulli_svc = state.services.tautulli_service;
    let discord_svc = state.services.discord_service;
    let discord_users_svc = state.services.discord_users_service;
    let discord_tokens_svc = state.services.discord_tokens_service;
    let plex_users_svc = state.services.plex_users_service;
    let plex_tokens_svc = state.services.plex_tokens_service;

    let resp = plex_svc
        .pin_claim(query_string.id, &query_string.code)
        .await?;

    let discord_token = session
        .get::<String>(DISCORD_CODE)
        .ok_or_else(|| anyhow!("no code found for session"))?;

    let is_subscriber = plex_svc
        .get_devices(&resp.auth_token)
        .await?
        .iter()
        .any(|d| d.client_identifier == state.config.plex.server_id);

    let token = discord_svc.token(&discord_token).await?;

    let d_access_token = String::from(token.access_token().secret());
    let discord_user = discord_svc.user(&d_access_token).await?;

    let plex_user = plex_svc.user(&resp.auth_token).await?;

    tracing::info!(
        "{} is a subscriber: {}",
        discord_user.username,
        is_subscriber
    );

    match discord_users_svc
        .create(&discord_user.id, &discord_user.username)
        .await
    {
        Ok(result) => match result {
            CreateDiscordUserResult::Ok(_user) => (),
            CreateDiscordUserResult::Error(err) => match err.error {
                CreateDiscordUserErrorVariant::UserAlreadyExists => (),
                CreateDiscordUserErrorVariant::InternalError => {
                    return Err(DisplexError(anyhow::anyhow!(
                        "failed to create discord_user: {:?}",
                        err
                    )))
                }
            },
        },
        Err(err) => {
            return Err(DisplexError(anyhow::anyhow!(
                "failed to create discord_user: {:?}",
                err
            )))
        }
    };
    let scopes: String = token.scopes().map_or("".into(), |d| {
        d.iter().map(|i| i.to_string() + ",").collect()
    });
    match discord_tokens_svc
        .create(
            token.access_token().secret(),
            token
                .refresh_token()
                .expect("expecting refresh token")
                .secret(),
            &(chrono::Utc::now()
                + chrono::Duration::seconds(
                    token
                        .expires_in()
                        .unwrap_or(Duration::from_secs(1800))
                        .as_secs() as i64,
                )),
            &scopes,
            &discord_user.id,
        )
        .await
    {
        Ok(result) => match result {
            CreateDiscordTokenResult::Ok(_) => (),
            CreateDiscordTokenResult::Error(err) => match err.error {
                CreateDiscordTokenErrorVariant::TokenAlreadyExists => (),
                CreateDiscordTokenErrorVariant::InternalError => {
                    return Err(DisplexError(anyhow::anyhow!(
                        "failed to create discord_token: {:?}",
                        err
                    )))
                }
            },
        },
        Err(err) => {
            return Err(DisplexError(anyhow::anyhow!(
                "failed to create discord_token: {:?}",
                err
            )))
        }
    };

    match plex_users_svc
        .create(
            plex_user.id,
            &plex_user.username,
            is_subscriber,
            &discord_user.id,
        )
        .await
    {
        Ok(result) => match result {
            CreatePlexUserResult::Ok(_) => (),
            CreatePlexUserResult::Error(err) => match err.error {
                CreatePlexUserErrorVariant::TokenAlreadyExists => (),
                CreatePlexUserErrorVariant::InternalError => {
                    return Err(DisplexError(anyhow::anyhow!(
                        "failed to create plex_user: {:?}",
                        err
                    )))
                }
            },
        },
        Err(err) => {
            return Err(DisplexError(anyhow::anyhow!(
                "failed to create plex_user: {:?}",
                err
            )))
        }
    };

    match plex_tokens_svc
        .create(&resp.auth_token, &plex_user.id)
        .await
    {
        Ok(result) => match result {
            CreatePlexTokenResult::Ok(_) => (),
            CreatePlexTokenResult::Error(err) => match err.error {
                CreatePlexTokenErrorVariant::TokenAlreadyExists => (),
                CreatePlexTokenErrorVariant::InternalError => {
                    return Err(DisplexError(anyhow::anyhow!(
                        "failed to create plex_token: {:?}",
                        err
                    )))
                }
            },
        },
        Err(err) => {
            return Err(DisplexError(anyhow::anyhow!(
                "failed to create plex_token: {:?}",
                err
            )))
        }
    };

    let mut data = ApplicationMetadataUpdate {
        platform_name: String::from(&state.config.application_name),
        metadata: ApplicationMetadata {
            is_subscriber,
            ..Default::default()
        },
        ..Default::default()
    };
    if is_subscriber {
        let watch_stats = tautulli_svc
            .get_user_watch_time_stats(plex_user.id, Some(true), Some(QueryDays::Total))
            .await?;

        if let Some(latest) = watch_stats.get(0) {
            data.metadata.total_watches = latest.total_plays;
            data.metadata.hours_watched = latest.total_time / 3600;
        };
    };

    discord_svc
        .link_application(state.config.discord.client_id, data, &d_access_token)
        .await?;
    Ok(Redirect::to(&format!(
        "discord://-/channels/{}/@home",
        state.config.discord.server_id
    )))
}

pub fn routes() -> Router<DisplexState> {
    Router::new().route("/plex/callback", get(callback))
}

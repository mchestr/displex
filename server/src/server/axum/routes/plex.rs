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
use cookie::Key;
use oauth2::TokenResponse;
use sea_orm::TransactionTrait;
use serde::Deserialize;
use tower_cookies::Cookies;

use crate::{
    errors::DisplexError,
    server::axum::{
        DisplexState,
        DISCORD_CODE,
    },
    services::{
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
        plex_token::resolver::{
            CreatePlexTokenErrorVariant,
            CreatePlexTokenResult,
        },
        plex_user::resolver::{
            CreatePlexUserErrorVariant,
            CreatePlexUserResult,
        },
        tautulli::models::QueryDays,
    },
};

#[derive(Deserialize)]
struct CallbackQueryParams {
    pub id: u64,
    pub code: String,
}

async fn callback(
    cookies: Cookies,
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
    let overseerr_svc = state.services.overseerr_service;

    let resp = plex_svc
        .pin_claim(query_string.id, &query_string.code)
        .await?;

    let key = Key::from(state.config.session.secret_key.as_bytes());
    let signed = cookies.signed(&key);
    let discord_token = signed
        .get(DISCORD_CODE)
        .ok_or_else(|| anyhow!("no code found for session"))?;

    let is_subscribed = plex_svc
        .get_devices(&resp.auth_token)
        .await?
        .iter()
        .any(|d| d.client_identifier == state.config.plex.server_id);

    let token = discord_svc.token(discord_token.value()).await?;
    let d_access_token = String::from(token.access_token().secret());
    let discord_user = discord_svc.user(&d_access_token).await?;
    let discord_user_id = String::from(&discord_user.id);
    let plex_user = plex_svc.user(&resp.auth_token).await?;

    tracing::info!(
        "{} is a subscriber: {}",
        discord_user.username,
        is_subscribed
    );

    state
        .services
        .db
        .transaction::<_, (), DisplexError>(|txn| {
            Box::pin(async move {
                let result = discord_users_svc
                    .create_with_conn(&discord_user.id, &discord_user.username, txn)
                    .await?;
                match result {
                    CreateDiscordUserResult::Error(err) => match err.error {
                        CreateDiscordUserErrorVariant::InternalError => {
                            Err(DisplexError(anyhow!("internal error")))
                        }
                    },
                    _ => Ok(()),
                }?;

                let scopes: String = token.scopes().map_or("".into(), |d| {
                    d.iter().map(|i| i.to_string() + ",").collect()
                });
                let result = discord_tokens_svc
                    .create_with_conn(
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
                        txn,
                    )
                    .await?;
                match result {
                    CreateDiscordTokenResult::Error(err) => match err.error {
                        CreateDiscordTokenErrorVariant::InternalError => {
                            Err(DisplexError(anyhow!("internal error")))
                        }
                    },
                    _ => Ok(()),
                }?;

                let result = plex_users_svc
                    .create_with_conn(
                        &plex_user.id.to_string(),
                        &plex_user.username,
                        is_subscribed,
                        &discord_user.id,
                        txn,
                    )
                    .await?;
                match result {
                    CreatePlexUserResult::Error(err) => match err.error {
                        CreatePlexUserErrorVariant::InternalError => {
                            Err(DisplexError(anyhow!("internal error")))
                        }
                    },
                    _ => Ok(()),
                }?;

                let result = plex_tokens_svc
                    .create_with_conn(&resp.auth_token, &plex_user.id.to_string(), txn)
                    .await?;
                match result {
                    CreatePlexTokenResult::Error(err) => match err.error {
                        CreatePlexTokenErrorVariant::InternalError => {
                            Err(DisplexError(anyhow!("internal error")))
                        }
                    },
                    _ => Ok(()),
                }?;
                Ok(())
            })
        })
        .await
        .expect("test");

    let mut data = ApplicationMetadataUpdate {
        platform_name: String::from(&state.config.application_name),
        metadata: ApplicationMetadata {
            is_subscribed,
            ..Default::default()
        },
        ..Default::default()
    };
    if is_subscribed {
        let watch_stats = tautulli_svc
            .get_user_watch_time_stats(
                &plex_user.id.to_string(),
                Some(true),
                Some(QueryDays::Total),
            )
            .await?;

        if let Some(latest) = watch_stats.first() {
            data.metadata.watched_hours = latest.total_time / 3600;
        };
    };

    discord_svc
        .link_application(state.config.discord.client_id, data, &d_access_token)
        .await?;
    overseerr_svc
        .verified_user(&discord_user_id, &plex_user.id.to_string())
        .await?;
    Ok(Redirect::to(&format!(
        "discord://-/channels/{}/@home",
        state.config.discord.server_id
    )))
}

pub fn routes() -> Router<DisplexState> {
    Router::new().route("/auth/plex/callback", get(callback))
}

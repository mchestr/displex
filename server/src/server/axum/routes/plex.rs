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
use sea_orm::TransactionTrait;
use serde::Deserialize;
use tower_cookies::Cookies;

use crate::{
    errors::DisplexError,
    server::{
        axum::DisplexState,
        cookies::{
            get_cookie_data,
            set_cookie_data,
        },
    },
    services::{
        discord::models::{
            ApplicationMetadata,
            ApplicationMetadataUpdate,
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
struct PlexAuthQueryParams {
    pub next: Option<String>,
}

async fn plex_auth(
    State(state): State<DisplexState>,
    query_string: Query<PlexAuthQueryParams>,
) -> Result<impl IntoResponse, DisplexError> {
    let pin = state.services.plex_service.get_pin().await?;
    let next = match &query_string.next {
        Some(next) => next.to_string(),
        None => String::new(),
    };
    tracing::debug!("next:{}", next);
    let url = state
        .services
        .plex_service
        .generate_auth_url(pin.id, &pin.code, &next)
        .await?;
    Ok(Redirect::to(&url))
}

#[derive(Deserialize)]
struct CallbackQueryParams {
    pub id: u64,
    pub code: String,
    pub next: Option<String>,
}

async fn callback(
    cookies: Cookies,
    State(state): State<DisplexState>,
    query_string: Query<CallbackQueryParams>,
) -> Result<impl IntoResponse, DisplexError> {
    let plex_svc = state.services.plex_service;
    let tautulli_svc = state.services.tautulli_service;
    let discord_svc = state.services.discord_service;
    let _discord_users_svc = state.services.discord_users_service;
    let discord_tokens_svc = state.services.discord_tokens_service;
    let plex_users_svc = state.services.plex_users_service;
    let plex_tokens_svc = state.services.plex_tokens_service;
    let overseerr_svc = state.services.overseerr_service;

    let resp = plex_svc
        .pin_claim(query_string.id, &query_string.code)
        .await?;

    let mut cookie_data = get_cookie_data(&state.config.session.secret_key, &cookies)?;
    let discord_token = discord_tokens_svc
        .latest_token(&cookie_data.discord_user.clone().unwrap())
        .await?
        .unwrap();
    let plex_user = plex_svc.user(&resp.auth_token).await?;
    let is_subscribed = plex_svc
        .get_devices(&resp.auth_token)
        .await?
        .iter()
        .any(|d| d.client_identifier == state.config.plex.server_id);

    tracing::info!(
        "{:?} is a subscriber: {}",
        cookie_data.discord_user,
        is_subscribed
    );

    state
        .services
        .db
        .transaction::<_, (), DisplexError>(|txn| {
            let discord_user_id = cookie_data.discord_user.clone();
            Box::pin(async move {
                let result = plex_users_svc
                    .create_with_conn(
                        &plex_user.id.to_string(),
                        &plex_user.username,
                        is_subscribed,
                        &discord_user_id.unwrap(),
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
        .expect("failed to save plex user info to database");

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

    cookie_data.plex_user = Some(plex_user.id.to_string());
    set_cookie_data(&state.config.session.secret_key, &cookies, &cookie_data)?;

    discord_svc
        .link_application(
            state.config.discord.client_id,
            data,
            &discord_token.access_token,
        )
        .await?;
    overseerr_svc
        .verified_user(
            &cookie_data.discord_user.unwrap(),
            &plex_user.id.to_string(),
        )
        .await?;

    let url = match &query_string.next {
        Some(next) => next,
        None => "/",
    };
    tracing::debug!("url:{}", url);
    Ok(Redirect::to(url))
}

pub fn routes() -> Router<DisplexState> {
    Router::new()
        .route("/auth/plex", get(plex_auth))
        .route("/auth/plex/callback", get(callback))
}
